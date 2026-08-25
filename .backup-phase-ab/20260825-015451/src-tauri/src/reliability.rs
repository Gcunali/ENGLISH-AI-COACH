use crate::{database, paths::LocalPaths, sha256};
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

pub const PLATFORM_RELIABILITY_SCHEMA_VERSION: u32 = 1;
pub const APP_BACKUP_FORMAT_VERSION: u32 = 1;
pub const DIAGNOSTIC_REPORT_VERSION: u32 = 1;
pub const STARTUP_RECOVERY_RULE_VERSION: u32 = 1;
pub const CURRENT_DATABASE_SCHEMA_VERSION: u32 = 19;
pub const SYSTEM_EVENT_SCHEMA_VERSION: u32 = 1;
pub const SYSTEM_EVENT_RETENTION: usize = 300;
const DATABASE_FILE: &str = "database.sqlite3";
const MANIFEST_FILE: &str = "manifest.json";
const PENDING_RESTORE_FILE: &str = "pending-restore.json";
const LAST_RESTORE_FILE: &str = "last-restore-result.json";

#[derive(Clone, Default)]
pub struct ReliabilityManager {
    operation: Arc<Mutex<OperationState>>,
}

#[derive(Clone, Debug, Default)]
struct OperationState {
    operation: String,
    error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupFileEntry {
    pub name: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupManifest {
    pub backup_format_version: u32,
    pub created_at: String,
    pub app_database_schema_version: u32,
    pub app_version: String,
    pub database_file: String,
    pub database_sha256: String,
    pub settings_included: bool,
    pub files: Vec<BackupFileEntry>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSummaryDto {
    pub backup_id: String,
    pub created_at: String,
    pub path: String,
    pub database_bytes: u64,
    pub database_sha256: String,
    pub schema_version: u32,
    pub valid: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupValidationDto {
    pub valid: bool,
    pub backup_id: String,
    pub schema_version: Option<u32>,
    pub integrity: String,
    pub foreign_key_violations: usize,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupStatusDto {
    pub operation: String,
    pub error: Option<String>,
    pub backup_directory: String,
    pub last_backup: Option<BackupSummaryDto>,
    pub restore_allowed: bool,
    pub restore_block_reason: Option<String>,
    pub pending_restart: bool,
    pub last_restore: Option<RestoreResultRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResultRecord {
    pub status: String,
    pub backup_id: String,
    pub message: String,
    pub occurred_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreScheduledDto {
    pub backup_id: String,
    pub safety_backup_id: String,
    pub restart_required: bool,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PendingRestore {
    backup_id: String,
    safety_backup_id: String,
    created_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemEventDto {
    pub id: String,
    pub severity: String,
    pub component: String,
    pub event_code: String,
    pub details: Option<Value>,
    pub occurred_at: String,
}

#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TempCleanupResult {
    pub inspected: usize,
    pub removed: usize,
    pub failed: usize,
    pub removed_names: Vec<String>,
}

impl ReliabilityManager {
    pub fn create_backup(&self, paths: &LocalPaths) -> Result<BackupSummaryDto, String> {
        let mut state = self
            .operation
            .try_lock()
            .map_err(|_| "Another data operation is already running.".to_owned())?;
        if !state.operation.is_empty()
            && state.operation != "completed"
            && state.operation != "failed"
        {
            return Err("Another data operation is already running.".into());
        }
        state.operation = "creating".into();
        state.error = None;
        let result = create_backup(paths, "EnglishAICoach-Backup");
        state.operation = if result.is_ok() {
            "completed"
        } else {
            "failed"
        }
        .into();
        state.error = result.as_ref().err().cloned();
        result
    }

    pub fn schedule_restore(
        &self,
        paths: &LocalPaths,
        backup_id: &str,
    ) -> Result<RestoreScheduledDto, String> {
        let mut state = self
            .operation
            .try_lock()
            .map_err(|_| "Another data operation is already running.".to_owned())?;
        if !state.operation.is_empty()
            && state.operation != "completed"
            && state.operation != "failed"
        {
            return Err("Another data operation is already running.".into());
        }
        state.operation = "validating".into();
        state.error = None;
        let result = (|| {
            validate_backup(paths, backup_id)?;
            state.operation = "creating_safety_backup".into();
            let safety = create_backup(paths, "pre-restore-safety-backup")?;
            state.operation = "restoring".into();
            let pending = PendingRestore {
                backup_id: backup_id.to_owned(),
                safety_backup_id: safety.backup_id.clone(),
                created_at: utc_now(&paths.db_file()),
            };
            atomic_write_json(&paths.reliability.join(PENDING_RESTORE_FILE), &pending)?;
            Ok(RestoreScheduledDto {
                backup_id: backup_id.to_owned(),
                safety_backup_id: safety.backup_id,
                restart_required: true,
                message:
                    "Backup validated and safety backup created. Restart the app to finish restore."
                        .into(),
            })
        })();
        state.operation = if result.is_ok() {
            "completed"
        } else {
            "failed"
        }
        .into();
        state.error = result.as_ref().err().cloned();
        result
    }

    pub fn status(
        &self,
        paths: &LocalPaths,
        restore_allowed: bool,
        reason: Option<String>,
    ) -> BackupStatusDto {
        let state = self
            .operation
            .lock()
            .expect("reliability operation lock")
            .clone();
        BackupStatusDto {
            operation: if state.operation.is_empty() {
                "idle".into()
            } else {
                state.operation
            },
            error: state.error,
            backup_directory: paths.backups.display().to_string(),
            last_backup: list_backups(paths).into_iter().next(),
            restore_allowed,
            restore_block_reason: reason,
            pending_restart: paths.reliability.join(PENDING_RESTORE_FILE).is_file(),
            last_restore: read_json(&paths.reliability.join(LAST_RESTORE_FILE)).ok(),
        }
    }
}

pub fn create_backup(paths: &LocalPaths, label: &str) -> Result<BackupSummaryDto, String> {
    fs::create_dir_all(&paths.backups).map_err(io("create backup directory"))?;
    let id = format!(
        "{label}-{}-{}.eacbackup",
        unix_seconds(),
        &uuid::Uuid::new_v4().to_string()[..8]
    );
    let final_dir = paths.backups.join(&id);
    let stage = paths.backups.join(format!(".{id}.partial"));
    if stage.exists() {
        fs::remove_dir_all(&stage).map_err(io("clean incomplete backup staging"))?;
    }
    fs::create_dir(&stage).map_err(io("create backup staging"))?;
    let result = (|| {
        let db = stage.join(DATABASE_FILE);
        consistent_snapshot(&paths.db_file(), &db)?;
        let validation = validate_database(&db)?;
        if validation.schema_version > CURRENT_DATABASE_SCHEMA_VERSION {
            return Err("The local database uses an unsupported future schema.".into());
        }
        let hash = sha256::file(&db)?;
        let bytes = fs::metadata(&db)
            .map_err(io("inspect backup database"))?
            .len();
        let manifest = BackupManifest {
            backup_format_version: APP_BACKUP_FORMAT_VERSION,
            created_at: utc_now(&paths.db_file()),
            app_database_schema_version: validation.schema_version,
            app_version: env!("CARGO_PKG_VERSION").into(),
            database_file: DATABASE_FILE.into(),
            database_sha256: hash.clone(),
            settings_included: true,
            files: vec![BackupFileEntry {
                name: DATABASE_FILE.into(),
                sha256: hash.clone(),
                bytes,
            }],
        };
        atomic_write_json(&stage.join(MANIFEST_FILE), &manifest)?;
        validate_backup_directory(&stage, Some(&id))?;
        fs::rename(&stage, &final_dir).map_err(io("publish validated backup"))?;
        Ok(BackupSummaryDto {
            backup_id: id,
            created_at: manifest.created_at,
            path: final_dir.display().to_string(),
            database_bytes: bytes,
            database_sha256: hash,
            schema_version: validation.schema_version,
            valid: true,
        })
    })();
    if result.is_err() && stage.exists() {
        let _ = fs::remove_dir_all(stage);
    }
    result
}

fn consistent_snapshot(source: &Path, destination: &Path) -> Result<(), String> {
    let connection = database::open(source)?;
    let path = destination.to_string_lossy();
    connection
        .execute("VACUUM INTO ?1", [path.as_ref()])
        .map_err(|e| format!("Could not create a consistent SQLite snapshot: {e}"))?;
    Ok(())
}

pub fn list_backups(paths: &LocalPaths) -> Vec<BackupSummaryDto> {
    let Ok(entries) = fs::read_dir(&paths.backups) else {
        return Vec::new();
    };
    let mut values: Vec<_> = entries
        .flatten()
        .filter_map(|entry| {
            let id = entry.file_name().to_string_lossy().to_string();
            if !id.ends_with(".eacbackup") || !entry.file_type().ok()?.is_dir() {
                return None;
            }
            validate_backup_directory(&entry.path(), Some(&id))
                .ok()
                .map(|(manifest, validation)| BackupSummaryDto {
                    backup_id: id,
                    created_at: manifest.created_at,
                    path: entry.path().display().to_string(),
                    database_bytes: manifest.files.first().map_or(0, |f| f.bytes),
                    database_sha256: manifest.database_sha256,
                    schema_version: validation.schema_version,
                    valid: true,
                })
        })
        .collect();
    values.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    values
}

pub fn validate_backup(paths: &LocalPaths, backup_id: &str) -> Result<BackupValidationDto, String> {
    let directory = controlled_backup_path(paths, backup_id)?;
    let (_, db) = validate_backup_directory(&directory, Some(backup_id))?;
    Ok(BackupValidationDto {
        valid: true,
        backup_id: backup_id.into(),
        schema_version: Some(db.schema_version),
        integrity: db.integrity,
        foreign_key_violations: db.foreign_key_violations,
        message: "Backup is valid and compatible.".into(),
    })
}

fn controlled_backup_path(paths: &LocalPaths, id: &str) -> Result<PathBuf, String> {
    let mut components = Path::new(id).components();
    if !id.ends_with(".eacbackup")
        || !matches!(components.next(), Some(Component::Normal(_)))
        || components.next().is_some()
    {
        return Err("Invalid backup identifier.".into());
    }
    let path = paths.backups.join(id);
    let metadata = fs::symlink_metadata(&path).map_err(|_| "Backup was not found.".to_owned())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("Backup container is not a regular directory.".into());
    }
    Ok(path)
}

fn validate_backup_directory(
    directory: &Path,
    expected_id: Option<&str>,
) -> Result<(BackupManifest, DatabaseValidation), String> {
    if fs::symlink_metadata(directory)
        .map_err(io("inspect backup container"))?
        .file_type()
        .is_symlink()
    {
        return Err("Backup container symlinks are not allowed.".into());
    }
    let allowed = [DATABASE_FILE, MANIFEST_FILE];
    for entry in fs::read_dir(directory).map_err(io("read backup container"))? {
        let entry = entry.map_err(io("read backup entry"))?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !allowed.contains(&name.as_str())
            || entry
                .file_type()
                .map_err(io("inspect backup entry"))?
                .is_symlink()
            || !entry
                .file_type()
                .map_err(io("inspect backup entry"))?
                .is_file()
        {
            return Err(format!("Backup contains an unexpected file: {name}"));
        }
    }
    let manifest_path = directory.join(MANIFEST_FILE);
    if !manifest_path.is_file() {
        return Err("Backup manifest is missing.".into());
    }
    let manifest: BackupManifest =
        read_json(&manifest_path).map_err(|_| "Backup manifest is invalid.".to_owned())?;
    if manifest.backup_format_version != APP_BACKUP_FORMAT_VERSION {
        return Err("Unsupported backup format version.".into());
    }
    if manifest.database_file != DATABASE_FILE {
        return Err("Backup database path is invalid.".into());
    }
    if manifest.files.len() != 1 || manifest.files[0].name != DATABASE_FILE {
        return Err("Backup file allowlist is invalid.".into());
    }
    let db = directory.join(DATABASE_FILE);
    if !db.is_file() {
        return Err("Backup database is missing.".into());
    }
    if fs::symlink_metadata(&db)
        .map_err(io("inspect backup database"))?
        .file_type()
        .is_symlink()
    {
        return Err("Backup database symlinks are not allowed.".into());
    }
    let actual = sha256::file(&db)?;
    if !actual.eq_ignore_ascii_case(&manifest.database_sha256)
        || !actual.eq_ignore_ascii_case(&manifest.files[0].sha256)
    {
        return Err("Backup database SHA-256 does not match the manifest.".into());
    }
    let size = fs::metadata(&db)
        .map_err(io("inspect backup database"))?
        .len();
    if size != manifest.files[0].bytes {
        return Err("Backup database size does not match the manifest.".into());
    }
    let validation = validate_database(&db)?;
    if validation.schema_version != manifest.app_database_schema_version {
        return Err("Backup schema version does not match the manifest.".into());
    }
    if validation.schema_version > CURRENT_DATABASE_SCHEMA_VERSION {
        return Err("This backup was created by a newer version of English AI Coach.".into());
    }
    if validation.schema_version == 0 {
        return Err("Backup database has no supported schema.".into());
    }
    let _ = expected_id;
    Ok((manifest, validation))
}

#[derive(Clone, Debug)]
pub struct DatabaseValidation {
    pub schema_version: u32,
    pub integrity: String,
    pub foreign_key_violations: usize,
}
pub fn validate_database(path: &Path) -> Result<DatabaseValidation, String> {
    let uri = format!(
        "file:{}?immutable=1",
        path.to_string_lossy().replace('\\', "/")
    );
    let connection = Connection::open_with_flags(
        uri,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|e| format!("Backup database cannot be opened: {e}"))?;
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(|e| format!("SQLite integrity check failed: {e}"))?;
    if integrity != "ok" {
        return Err(format!("SQLite integrity check failed: {integrity}"));
    }
    let foreign_key_violations: usize = connection
        .prepare("PRAGMA foreign_key_check")
        .and_then(|mut s| s.query_map([], |_| Ok(()))?.collect::<Result<Vec<_>, _>>())
        .map_err(|e| format!("Foreign-key check failed: {e}"))?
        .len();
    if foreign_key_violations > 0 {
        return Err("Backup database contains foreign-key violations.".into());
    }
    for table in ["schema_migration", "settings", "conversation_exchange"] {
        let exists: i64 = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
                [table],
                |r| r.get(0),
            )
            .map_err(|e| format!("Could not inspect required tables: {e}"))?;
        if exists != 1 {
            return Err(format!(
                "Backup database is missing required table {table}."
            ));
        }
    }
    let schema_version: i64 = connection
        .query_row(
            "SELECT COALESCE(MAX(version),0) FROM schema_migration",
            [],
            |r| r.get(0),
        )
        .map_err(|e| format!("Could not read schema version: {e}"))?;
    Ok(DatabaseValidation {
        schema_version: schema_version.max(0) as u32,
        integrity,
        foreign_key_violations,
    })
}

pub fn process_pending_restore(paths: &LocalPaths) -> Result<Option<RestoreResultRecord>, String> {
    let pending_path = paths.reliability.join(PENDING_RESTORE_FILE);
    if !pending_path.is_file() {
        return Ok(None);
    }
    let pending: PendingRestore = read_json(&pending_path)
        .map_err(|_| "Pending restore instruction is invalid.".to_owned())?;
    fs::remove_file(&pending_path).map_err(io("consume pending restore instruction"))?;
    let result = apply_restore(paths, &pending);
    let record = match &result {
        Ok(()) => RestoreResultRecord {
            status: "completed".into(),
            backup_id: pending.backup_id.clone(),
            message: "Learning data restored successfully.".into(),
            occurred_at: utc_now(&paths.db_file()),
        },
        Err(error) => RestoreResultRecord {
            status: "failed".into(),
            backup_id: pending.backup_id.clone(),
            message: error.clone(),
            occurred_at: utc_now(&paths.db_file()),
        },
    };
    let _ = atomic_write_json(&paths.reliability.join(LAST_RESTORE_FILE), &record);
    result.map(|_| Some(record))
}

fn apply_restore(paths: &LocalPaths, pending: &PendingRestore) -> Result<(), String> {
    apply_restore_with(paths, pending, |current| {
        database::migrate(current)?;
        let db = validate_database(current)?;
        if db.schema_version != CURRENT_DATABASE_SCHEMA_VERSION {
            return Err("Restored database could not be migrated to the current schema.".into());
        }
        Ok(())
    })
}

fn apply_restore_with<F>(
    paths: &LocalPaths,
    pending: &PendingRestore,
    verify_after_swap: F,
) -> Result<(), String>
where
    F: FnOnce(&Path) -> Result<(), String>,
{
    validate_backup(paths, &pending.backup_id)?;
    validate_backup(paths, &pending.safety_backup_id)
        .map_err(|e| format!("Pre-restore safety backup is unavailable: {e}"))?;
    let source = controlled_backup_path(paths, &pending.backup_id)?.join(DATABASE_FILE);
    let stage = paths
        .database
        .join(format!("restore-stage-{}.sqlite3", uuid::Uuid::new_v4()));
    fs::copy(&source, &stage).map_err(io("stage restored database"))?;
    validate_database(&stage)?;
    let current = paths.db_file();
    let rollback = paths.database.join("restore-rollback.sqlite3");
    if rollback.exists() {
        fs::remove_file(&rollback).map_err(io("remove stale restore rollback"))?
    }
    remove_sqlite_sidecars(&current);
    if current.exists() {
        fs::rename(&current, &rollback).map_err(io("preserve current database for rollback"))?
    }
    let swapped =
        fs::rename(&stage, &current).map_err(io("replace database with restored staging"));
    if let Err(error) = swapped {
        if rollback.exists() {
            let _ = fs::rename(&rollback, &current);
        }
        let _ = fs::remove_file(&stage);
        return Err(error);
    }
    let verified = verify_after_swap(&current);
    if let Err(error) = verified {
        let _ = fs::remove_file(&current);
        if rollback.exists() {
            fs::rename(&rollback, &current).map_err(io("roll back failed restore"))?
        }
        return Err(format!(
            "Restore failed and the previous database was recovered: {error}"
        ));
    }
    if rollback.exists() {
        fs::remove_file(rollback).map_err(io("remove completed restore rollback"))?
    }
    Ok(())
}

fn remove_sqlite_sidecars(database: &Path) {
    for suffix in ["-wal", "-shm"] {
        let _ = fs::remove_file(format!("{}{}", database.display(), suffix));
    }
}

pub fn cleanup_owned_temp(paths: &LocalPaths) -> TempCleanupResult {
    let mut result = TempCleanupResult::default();
    let Ok(entries) = fs::read_dir(&paths.temporary_audio) else {
        return result;
    };
    for entry in entries.flatten() {
        result.inspected += 1;
        let Ok(kind) = entry.file_type() else {
            result.failed += 1;
            continue;
        };
        if !kind.is_file() || kind.is_symlink() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !is_owned_temp_name(&name) {
            continue;
        }
        match fs::remove_file(entry.path()) {
            Ok(()) => {
                result.removed += 1;
                result.removed_names.push(name)
            }
            Err(_) => result.failed += 1,
        }
    }
    result
}
fn is_owned_temp_name(name: &str) -> bool {
    if name.starts_with("placement-")
        || name.starts_with("pronunciation-")
        || name.starts_with("voice-n-")
        || name.starts_with("guided-lesson-")
    {
        return matches!(
            Path::new(name).extension().and_then(|x| x.to_str()),
            Some("wav" | "txt")
        );
    }
    let suffix = ["-input.wav", "-teacher.wav", "-transcript.txt"]
        .iter()
        .any(|s| name.ends_with(s));
    suffix
        && name
            .split('-')
            .take(5)
            .collect::<Vec<_>>()
            .join("-")
            .parse::<uuid::Uuid>()
            .is_ok()
}

pub fn record_event(
    database: &Path,
    severity: &str,
    component: &str,
    event_code: &str,
    details: Option<Value>,
) -> Result<(), String> {
    let allowed_severity = matches!(severity, "warning" | "error" | "recovery");
    if !allowed_severity {
        return Err("Invalid system-event severity.".into());
    }
    let allowed = matches!(
        event_code,
        "DB_INTEGRITY_FAILED"
            | "OLLAMA_UNAVAILABLE"
            | "WHISPER_MODEL_MISSING"
            | "PIPER_UNAVAILABLE"
            | "PRONUNCIATION_MODEL_INVALID"
            | "VOICE_WORKER_CRASHED"
            | "PRONUNCIATION_WORKER_CRASHED"
            | "TEMP_CLEANUP_FAILED"
            | "STALE_SESSION_RECOVERED"
            | "BACKUP_FAILED"
            | "RESTORE_FAILED"
    );
    if !allowed {
        return Err("Invalid system-event code.".into());
    }
    if let Some(value) = &details {
        let object = value
            .as_object()
            .ok_or_else(|| "System-event details must be an object.".to_owned())?;
        let fields = [
            "component",
            "available",
            "expectedVersion",
            "actualVersion",
            "recoveredCount",
            "failedCount",
        ];
        if object.keys().any(|key| !fields.contains(&key.as_str())) {
            return Err("System-event details contain a non-allowlisted field.".into());
        }
    }
    let connection = database::open(database)?;
    let now = utc_now(database);
    connection.execute("INSERT INTO app_system_event(id,event_schema_version,severity,component,event_code,details_json,occurred_at,created_at) VALUES(?1,1,?2,?3,?4,?5,?6,?6)",rusqlite::params![uuid::Uuid::new_v4().to_string(),severity,component,event_code,details.map(|v|v.to_string()),now]).map_err(|e|format!("Could not save system event: {e}"))?;
    connection.execute("DELETE FROM app_system_event WHERE id NOT IN (SELECT id FROM app_system_event ORDER BY occurred_at DESC,id DESC LIMIT ?1)",[SYSTEM_EVENT_RETENTION as i64]).map_err(|e|format!("Could not enforce system-event retention: {e}"))?;
    Ok(())
}
pub fn list_events(database: &Path, limit: u32) -> Result<Vec<SystemEventDto>, String> {
    let connection = database::open(database)?;
    let mut statement=connection.prepare("SELECT id,severity,component,event_code,details_json,occurred_at FROM app_system_event ORDER BY occurred_at DESC,id DESC LIMIT ?1").map_err(|e|e.to_string())?;
    let events = statement
        .query_map([limit.min(100) as i64], |r| {
            let raw: Option<String> = r.get(4)?;
            Ok(SystemEventDto {
                id: r.get(0)?,
                severity: r.get(1)?,
                component: r.get(2)?,
                event_code: r.get(3)?,
                details: raw.and_then(|x| serde_json::from_str(&x).ok()),
                occurred_at: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(events)
}

fn atomic_write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let temp = path.with_extension(format!("tmp-{}", uuid::Uuid::new_v4()));
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    fs::write(&temp, bytes).map_err(io("write temporary metadata"))?;
    fs::rename(temp, path).map_err(io("publish metadata"))
}
fn read_json<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T, String> {
    let bytes = fs::read(path).map_err(io("read JSON"))?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn utc_now(database: &Path) -> String {
    database::open(database)
        .and_then(|c| {
            c.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now')", [], |r| {
                r.get(0)
            })
            .map_err(|e| e.to_string())
        })
        .unwrap_or_else(|_| format!("unix:{}", unix_seconds()))
}
fn io(action: &'static str) -> impl Fn(std::io::Error) -> String {
    move |e| format!("Could not {action}: {e}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn test_paths() -> (PathBuf, LocalPaths) {
        let root = std::env::temp_dir().join(format!("eac-reliability-{}", uuid::Uuid::new_v4()));
        let paths = LocalPaths::create(root.clone()).unwrap();
        database::migrate(&paths.db_file()).unwrap();
        (root, paths)
    }
    fn bundle_database(paths: &LocalPaths, id: &str, source: &Path) -> String {
        let directory = paths.backups.join(id);
        fs::create_dir_all(&directory).unwrap();
        let database_file = directory.join(DATABASE_FILE);
        fs::copy(source, &database_file).unwrap();
        let validation = validate_database(&database_file).unwrap();
        let hash = sha256::file(&database_file).unwrap();
        let bytes = fs::metadata(&database_file).unwrap().len();
        let manifest = BackupManifest {
            backup_format_version: APP_BACKUP_FORMAT_VERSION,
            created_at: "2026-08-23T00:00:00Z".into(),
            app_database_schema_version: validation.schema_version,
            app_version: "test".into(),
            database_file: DATABASE_FILE.into(),
            database_sha256: hash.clone(),
            settings_included: true,
            files: vec![BackupFileEntry {
                name: DATABASE_FILE.into(),
                sha256: hash,
                bytes,
            }],
        };
        atomic_write_json(&directory.join(MANIFEST_FILE), &manifest).unwrap();
        id.into()
    }
    #[test]
    fn backup_manifest_hash_and_two_independent_backups_are_valid() {
        let (root, paths) = test_paths();
        let wal_writer = database::open(&paths.db_file()).unwrap();
        wal_writer
            .execute_batch("PRAGMA wal_autocheckpoint=0;")
            .unwrap();
        wal_writer.execute("INSERT INTO conversation_exchange(id,student_text,teacher_text) VALUES('wal','hello','hi')",[]).unwrap();
        let a = create_backup(&paths, "test").unwrap();
        let b = create_backup(&paths, "test").unwrap();
        assert_ne!(a.backup_id, b.backup_id);
        assert!(validate_backup(&paths, &a.backup_id).unwrap().valid);
        assert!(validate_backup(&paths, &b.backup_id).unwrap().valid);
        assert_eq!(
            database::open(&paths.db_file())
                .unwrap()
                .query_row("SELECT COUNT(*) FROM conversation_exchange", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
        drop(wal_writer);
        fs::remove_dir_all(root).unwrap()
    }
    #[test]
    fn rejects_missing_manifest_bad_version_missing_db_and_bad_hash() {
        let (root, paths) = test_paths();
        let backup = create_backup(&paths, "test").unwrap();
        let dir = paths.backups.join(&backup.backup_id);
        fs::remove_file(dir.join(MANIFEST_FILE)).unwrap();
        assert!(validate_backup(&paths, &backup.backup_id).is_err());
        fs::remove_dir_all(&dir).unwrap();
        let backup = create_backup(&paths, "test").unwrap();
        let dir = paths.backups.join(&backup.backup_id);
        let mut manifest: BackupManifest = read_json(&dir.join(MANIFEST_FILE)).unwrap();
        manifest.backup_format_version = 99;
        atomic_write_json(&dir.join(MANIFEST_FILE), &manifest).unwrap();
        assert!(validate_backup(&paths, &backup.backup_id).is_err());
        manifest.backup_format_version = 1;
        atomic_write_json(&dir.join(MANIFEST_FILE), &manifest).unwrap();
        fs::remove_file(dir.join(DATABASE_FILE)).unwrap();
        assert!(validate_backup(&paths, &backup.backup_id).is_err());
        fs::remove_dir_all(&dir).unwrap();
        let backup = create_backup(&paths, "test").unwrap();
        fs::write(
            paths.backups.join(&backup.backup_id).join(DATABASE_FILE),
            b"corrupt",
        )
        .unwrap();
        assert!(validate_backup(&paths, &backup.backup_id).is_err());
        fs::remove_dir_all(root).unwrap()
    }
    #[test]
    fn blocks_traversal_and_does_not_follow_symlink_like_paths() {
        let (root, paths) = test_paths();
        assert!(validate_backup(&paths, "..\\outside.eacbackup").is_err());
        assert!(validate_backup(&paths, "../outside.eacbackup").is_err());
        fs::remove_dir_all(root).unwrap()
    }
    #[test]
    fn cleanup_only_removes_owned_private_temp_files() {
        let (root, paths) = test_paths();
        fs::write(paths.temporary_audio.join("placement-a.wav"), b"x").unwrap();
        fs::write(paths.temporary_audio.join("personal.wav"), b"x").unwrap();
        fs::write(paths.models.join("model.wav"), b"x").unwrap();
        fs::create_dir_all(paths.backups.join("old.eacbackup")).unwrap();
        let result = cleanup_owned_temp(&paths);
        assert_eq!(result.removed, 1);
        assert!(paths.temporary_audio.join("personal.wav").exists());
        assert!(paths.models.join("model.wav").exists());
        assert!(paths.backups.join("old.eacbackup").exists());
        fs::remove_dir_all(root).unwrap()
    }
    #[test]
    fn event_details_are_allowlisted_and_retention_is_bounded() {
        let (root, paths) = test_paths();
        assert!(record_event(
            &paths.db_file(),
            "warning",
            "db",
            "DB_INTEGRITY_FAILED",
            Some(json!({"transcript":"secret"}))
        )
        .is_err());
        for i in 0..305 {
            record_event(
                &paths.db_file(),
                "warning",
                "temp",
                "TEMP_CLEANUP_FAILED",
                Some(json!({"failedCount":i})),
            )
            .unwrap();
        }
        let count: i64 = database::open(&paths.db_file())
            .unwrap()
            .query_row("SELECT COUNT(*) FROM app_system_event", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, SYSTEM_EVENT_RETENTION as i64);
        fs::remove_dir_all(root).unwrap()
    }
    #[test]
    fn current_version_restore_round_trip_creates_safety_backup_and_preserves_exact_state() {
        let (root, paths) = test_paths();
        let connection = database::open(&paths.db_file()).unwrap();
        connection.execute("INSERT INTO conversation_exchange(id,student_text,teacher_text) VALUES('state-a','A','A')",[]).unwrap();
        drop(connection);
        let original = create_backup(&paths, "test-current").unwrap();
        database::open(&paths.db_file()).unwrap().execute("INSERT INTO conversation_exchange(id,student_text,teacher_text) VALUES('state-b','B','B')",[]).unwrap();
        let manager = ReliabilityManager::default();
        let scheduled = manager
            .schedule_restore(&paths, &original.backup_id)
            .unwrap();
        assert!(scheduled.restart_required);
        assert!(paths.backups.join(&scheduled.safety_backup_id).is_dir());
        let result = process_pending_restore(&paths).unwrap().unwrap();
        assert_eq!(result.status, "completed");
        let connection = database::open(&paths.db_file()).unwrap();
        let ids: Vec<String> = connection
            .prepare("SELECT id FROM conversation_exchange ORDER BY id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(ids, vec!["state-a"]);
        assert_eq!(
            validate_database(&paths.db_file()).unwrap().schema_version,
            19
        );
        drop(connection);
        fs::remove_dir_all(root).unwrap()
    }
    #[test]
    fn backup_restore_preserves_guided_phase_s_runtime_state() {
        let (root, paths) = test_paths();
        let connection = database::open(&paths.db_file()).unwrap();
        connection.execute("INSERT INTO interactive_lesson_session(id,lesson_id,lesson_content_version,package_schema_version,lesson_flow_version,package_hash,engine_version,snapshot_version,status,stage_count,current_stage_index,package_snapshot_json,student_context_snapshot_json,started_at,updated_at) VALUES('guided-s','fixture',1,1,1,?1,1,1,'in_progress',1,0,'{}','{}','now','now')",["a".repeat(64)]).unwrap();
        connection.execute("INSERT INTO interactive_lesson_stage_state(id,session_id,stage_id,sequence_index,stage_type,stage_schema_version,required,status,attempt_count,started_at,updated_at) VALUES('stage-s','guided-s','listen',0,'listening',1,1,'active',0,'now','now')",[]).unwrap();
        connection.execute("INSERT INTO interactive_lesson_stage_runtime_state(session_id,stage_id,runtime_state_schema_version,state_json,updated_at) VALUES('guided-s','listen',1,'{\"kind\":\"listening\",\"segments\":[{\"segmentId\":\"one\",\"completedPlaybackCount\":1}]}','now')",[]).unwrap();
        drop(connection);
        let backup = create_backup(&paths, "guided-phase-s").unwrap();
        database::open(&paths.db_file())
            .unwrap()
            .execute(
                "DELETE FROM interactive_lesson_session WHERE id='guided-s'",
                [],
            )
            .unwrap();
        let manager = ReliabilityManager::default();
        manager.schedule_restore(&paths, &backup.backup_id).unwrap();
        process_pending_restore(&paths).unwrap().unwrap();
        let restored=database::open(&paths.db_file()).unwrap().query_row("SELECT COUNT(*) FROM interactive_lesson_stage_runtime_state WHERE session_id='guided-s' AND json_extract(state_json,'$.segments[0].completedPlaybackCount')=1",[],|row|row.get::<_,i64>(0)).unwrap();
        assert_eq!(restored, 1);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn backup_restore_preserves_guided_phase_t_exercise_progress_and_selection() {
        let (root, paths) = test_paths();
        let connection = database::open(&paths.db_file()).unwrap();
        connection.execute("INSERT INTO interactive_lesson_session(id,lesson_id,lesson_content_version,package_schema_version,lesson_flow_version,package_hash,engine_version,snapshot_version,status,stage_count,current_stage_index,package_snapshot_json,student_context_snapshot_json,started_at,updated_at) VALUES('guided-t','fixture',1,1,1,?1,1,1,'in_progress',1,0,'{}','{}','now','now')",["b".repeat(64)]).unwrap();
        connection.execute("INSERT INTO interactive_lesson_stage_state(id,session_id,stage_id,sequence_index,stage_type,stage_schema_version,required,status,attempt_count,started_at,updated_at) VALUES('stage-t','guided-t','exercise',0,'exercise',1,1,'active',0,'now','now')",[]).unwrap();
        connection.execute("INSERT INTO interactive_lesson_stage_runtime_state(session_id,stage_id,runtime_state_schema_version,state_json,updated_at) VALUES('guided-t','exercise',1,'{\"kind\":\"exercise\",\"currentExerciseIndex\":1,\"items\":[{\"exerciseId\":\"one\",\"selectedAttemptId\":\"attempt-t\",\"attemptCount\":2}]}','now')",[]).unwrap();
        connection.execute("INSERT INTO interactive_lesson_exercise_attempt(id,submission_id,session_id,stage_id,exercise_id,exercise_type,attempt_index,response_schema_version,response_json,result_schema_version,result_json,correct,selected,submitted_at,selected_at,created_at) VALUES('attempt-t','submit-t','guided-t','exercise','one','short_answer_exact',2,1,'{\"exerciseType\":\"short_answer_exact\",\"value\":{\"text\":\"wrong\"}}',1,'{\"schemaVersion\":1,\"correct\":false,\"feedback\":\"Review\",\"explanation\":null,\"expectedAnswer\":{\"kind\":\"short_answer_exact\",\"answer\":\"right\"},\"normalizationVersion\":1}',0,1,'now','now','now')",[]).unwrap();
        connection.execute("INSERT INTO interactive_lesson_guided_conversation_turn(id,event_id,session_id,stage_id,sequence_index,role,text,text_schema_version,word_count,partial,created_at,committed_at) VALUES('turn-u','event-u','guided-t','exercise',0,'student','Hello teacher',1,2,0,'now','now')",[]).unwrap();
        drop(connection);
        let backup = create_backup(&paths, "guided-phase-t").unwrap();
        database::open(&paths.db_file())
            .unwrap()
            .execute(
                "DELETE FROM interactive_lesson_session WHERE id='guided-t'",
                [],
            )
            .unwrap();
        let manager = ReliabilityManager::default();
        manager.schedule_restore(&paths, &backup.backup_id).unwrap();
        process_pending_restore(&paths).unwrap().unwrap();
        let restored = database::open(&paths.db_file()).unwrap();
        let count:i64=restored.query_row("SELECT COUNT(*) FROM interactive_lesson_exercise_attempt WHERE id='attempt-t' AND selected=1",[],|row|row.get(0)).unwrap();
        let current:i64=restored.query_row("SELECT json_extract(state_json,'$.currentExerciseIndex') FROM interactive_lesson_stage_runtime_state WHERE session_id='guided-t'",[],|row|row.get(0)).unwrap();
        let guided_turns:i64=restored.query_row("SELECT COUNT(*) FROM interactive_lesson_guided_conversation_turn WHERE id='turn-u' AND text='Hello teacher'",[],|row|row.get(0)).unwrap();
        assert_eq!((count, current, guided_turns), (1, 1, 1));
        drop(restored);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn backup_restore_preserves_interactive_lesson_analysis_exactly() {
        let (root, paths) = test_paths();
        let connection = database::open(&paths.db_file()).unwrap();
        connection.execute("INSERT INTO interactive_lesson_session(id,lesson_id,lesson_content_version,package_schema_version,lesson_flow_version,package_hash,engine_version,snapshot_version,status,stage_count,current_stage_index,package_snapshot_json,student_context_snapshot_json,started_at,updated_at) VALUES('guided-v','fixture',1,1,1,?1,1,1,'completed',1,0,'{}','{}','now','now')", ["c".repeat(64)]).unwrap();
        connection.execute("INSERT INTO interactive_lesson_stage_state(id,session_id,stage_id,sequence_index,stage_type,stage_schema_version,required,status,attempt_count,started_at,completed_at,updated_at) VALUES('stage-v','guided-v','analysis',0,'analysis',1,1,'completed',0,'now','now','now')", []).unwrap();
        connection.execute("INSERT INTO interactive_lesson_analysis(id,session_id,stage_id,analysis_schema_version,analysis_engine_version,evidence_schema_version,conversation_evaluator_version,conversation_prompt_version,model_id,evidence_hash,evidence_json,conversation_status,conversation_result_json,final_result_json,status,created_at,updated_at,finalized_at) VALUES('analysis-v','guided-v','analysis',1,1,1,1,1,'qwen3.5:4b',?1,'{\"schemaVersion\":1,\"sessionId\":\"guided-v\"}','completed','{\"grammarScore\":80}','{\"schemaVersion\":1,\"status\":\"completed\",\"participation\":{\"completedStages\":1}}','completed','now','now','now')", ["d".repeat(64)]).unwrap();
        let expected: (String, String, String, String) = connection
            .query_row(
                "SELECT evidence_hash,evidence_json,conversation_result_json,final_result_json FROM interactive_lesson_analysis WHERE id='analysis-v'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        drop(connection);

        let backup = create_backup(&paths, "guided-phase-v-analysis").unwrap();
        database::open(&paths.db_file())
            .unwrap()
            .execute(
                "UPDATE interactive_lesson_analysis SET final_result_json='{\"mutated\":true}' WHERE id='analysis-v'",
                [],
            )
            .unwrap();
        let manager = ReliabilityManager::default();
        manager.schedule_restore(&paths, &backup.backup_id).unwrap();
        process_pending_restore(&paths).unwrap().unwrap();

        let restored = database::open(&paths.db_file()).unwrap();
        let actual: (String, String, String, String) = restored
            .query_row(
                "SELECT evidence_hash,evidence_json,conversation_result_json,final_result_json FROM interactive_lesson_analysis WHERE id='analysis-v'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(actual, expected);
        drop(restored);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn valid_older_schema_restores_then_migrates_and_future_schema_is_rejected() {
        let (root, paths) = test_paths();
        let old = root.join("old.sqlite3");
        let connection = Connection::open(&old).unwrap();
        connection
            .execute_batch(include_str!("../migrations/001_initial.sql"))
            .unwrap();
        connection.execute("INSERT INTO conversation_exchange(id,student_text,teacher_text) VALUES('old','old','old')",[]).unwrap();
        drop(connection);
        let old_id = bundle_database(&paths, "old-schema.eacbackup", &old);
        let safety = create_backup(&paths, "safety").unwrap();
        apply_restore(
            &paths,
            &PendingRestore {
                backup_id: old_id,
                safety_backup_id: safety.backup_id,
                created_at: "now".into(),
            },
        )
        .unwrap();
        assert_eq!(
            validate_database(&paths.db_file()).unwrap().schema_version,
            19
        );
        let future = root.join("future.sqlite3");
        fs::copy(&paths.db_file(), &future).unwrap();
        database::open(&future)
            .unwrap()
            .execute("INSERT INTO schema_migration(version) VALUES(99)", [])
            .unwrap();
        let future_id = bundle_database(&paths, "future-schema.eacbackup", &future);
        let error = validate_backup(&paths, &future_id).unwrap_err();
        assert!(error.contains("newer version"));
        fs::remove_dir_all(root).unwrap()
    }
    #[test]
    fn foreign_key_violation_is_rejected_before_swap() {
        let (root, paths) = test_paths();
        let invalid = root.join("invalid-fk.sqlite3");
        fs::copy(paths.db_file(), &invalid).unwrap();
        let connection = Connection::open(&invalid).unwrap();
        connection.execute_batch("PRAGMA foreign_keys=OFF; CREATE TABLE fk_parent(id TEXT PRIMARY KEY); CREATE TABLE fk_child(parent_id TEXT REFERENCES fk_parent(id)); INSERT INTO fk_child(parent_id) VALUES('missing');").unwrap();
        drop(connection);
        let id = "invalid-fk.eacbackup";
        let dir = paths.backups.join(id);
        fs::create_dir_all(&dir).unwrap();
        let db = dir.join(DATABASE_FILE);
        fs::copy(&invalid, &db).unwrap();
        let hash = sha256::file(&db).unwrap();
        let bytes = fs::metadata(&db).unwrap().len();
        let manifest = BackupManifest {
            backup_format_version: 1,
            created_at: "now".into(),
            app_database_schema_version: 13,
            app_version: "test".into(),
            database_file: DATABASE_FILE.into(),
            database_sha256: hash.clone(),
            settings_included: true,
            files: vec![BackupFileEntry {
                name: DATABASE_FILE.into(),
                sha256: hash,
                bytes,
            }],
        };
        atomic_write_json(&dir.join(MANIFEST_FILE), &manifest).unwrap();
        assert!(validate_backup(&paths, id)
            .unwrap_err()
            .contains("foreign-key"));
        fs::remove_dir_all(root).unwrap()
    }
    #[test]
    fn post_swap_failure_rolls_back_to_the_previous_database() {
        let (root, paths) = test_paths();
        database::open(&paths.db_file()).unwrap().execute("INSERT INTO conversation_exchange(id,student_text,teacher_text) VALUES('backup-state','x','x')",[]).unwrap();
        let backup = create_backup(&paths, "rollback-source").unwrap();
        database::open(&paths.db_file()).unwrap().execute("INSERT INTO conversation_exchange(id,student_text,teacher_text) VALUES('current-state','y','y')",[]).unwrap();
        let safety = create_backup(&paths, "rollback-safety").unwrap();
        let pending = PendingRestore {
            backup_id: backup.backup_id,
            safety_backup_id: safety.backup_id,
            created_at: "now".into(),
        };
        let error =
            apply_restore_with(
                &paths,
                &pending,
                |_| Err("induced post-swap failure".into()),
            )
            .unwrap_err();
        assert!(error.contains("previous database was recovered"));
        let current: i64 = database::open(&paths.db_file())
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM conversation_exchange WHERE id='current-state'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(current, 1);
        fs::remove_dir_all(root).unwrap()
    }
    #[test]
    #[ignore = "manual physical backup and copy-only restore validation of the user's local database"]
    fn physical_human_database_backup_and_copy_restore_validation() {
        let root = PathBuf::from(std::env::var_os("LOCALAPPDATA").expect("LOCALAPPDATA"))
            .join("com.englishaicoach.desktop");
        let paths = LocalPaths::create(root).unwrap();
        let before = validate_database(&paths.db_file()).unwrap();
        let backup = create_backup(&paths, "EnglishAICoach-Physical-Backup").unwrap();
        let verified = validate_backup(&paths, &backup.backup_id).unwrap();
        let temp_root = std::env::temp_dir().join(format!(
            "eac-physical-restore-copy-{}",
            uuid::Uuid::new_v4()
        ));
        let temp_paths = LocalPaths::create(temp_root.clone()).unwrap();
        let source = controlled_backup_path(&paths, &backup.backup_id)
            .unwrap()
            .join(DATABASE_FILE);
        fs::copy(source, temp_paths.db_file()).unwrap();
        database::migrate(&temp_paths.db_file()).unwrap();
        let copy = validate_database(&temp_paths.db_file()).unwrap();
        println!("{}",serde_json::to_string_pretty(&json!({"backup":backup,"validation":verified,"sourceSchema":before.schema_version,"copySchema":copy.schema_version,"copyIntegrity":copy.integrity,"copyForeignKeys":copy.foreign_key_violations})).unwrap());
        fs::remove_dir_all(temp_root).unwrap();
    }
}
