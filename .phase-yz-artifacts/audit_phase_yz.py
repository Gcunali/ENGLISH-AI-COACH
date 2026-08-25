import hashlib
import json
import sqlite3
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(r"C:\ENGLISH AI COACH")
LESSONS = ROOT / "src-tauri" / "resources" / "interactive-lessons"
CURRICULUM = ROOT / "src-tauri" / "resources" / "curriculum" / "english-core" / "curriculum.json"
DB = Path(r"C:\Users\guicu\AppData\Local\com.englishaicoach.desktop\database\english-ai-coach.sqlite3")
BACKUP = ROOT / ".backup-phase-yz" / "20260825-005857"
OUT = ROOT / ".phase-z-artifacts"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def theory_words(package):
    total = 0
    for block in package["stages"][0]["payload"]["blocks"]:
        total += len(str(block.get("text", "")).split())
        total += sum(len(item.split()) for item in block.get("items", []))
    return total


packages = []
for path in sorted(LESSONS.glob("*-v1/lesson.json")):
    package = json.loads(path.read_text(encoding="utf-8"))
    package["_path"] = path
    packages.append(package)

curriculum = json.loads(CURRICULUM.read_text(encoding="utf-8"))
refs = [(ref["lessonId"], ref["contentVersion"]) for level in curriculum["levels"] for unit in level["units"] for ref in unit["lessons"]]
available = {(p["lessonId"], p["contentVersion"]) for p in packages}

levels = defaultdict(list)
for package in packages:
    levels[package["cefrBand"]].append(package)

metrics = {}
for level, values in sorted(levels.items()):
    metrics[level] = {
        "lessons": len(values),
        "published": sum(p["publicationState"] == "published" for p in values),
        "theoryWords": {"min": min(map(theory_words, values)), "max": max(map(theory_words, values)), "average": round(sum(map(theory_words, values)) / len(values), 1)},
        "averageVocabulary": round(sum(len(p["stages"][1]["payload"]["items"]) for p in values) / len(values), 1),
        "averageListening": round(sum(len(p["stages"][2]["payload"]["segments"]) for p in values) / len(values), 1),
        "averageExercises": round(sum(len(p["stages"][5]["payload"]["items"]) for p in values) / len(values), 1),
        "conversationTurns": sorted({(p["stages"][6]["payload"]["minimumStudentTurns"], p["stages"][6]["payload"]["recommendedStudentTurns"], p["stages"][6]["payload"]["maximumStudentTurns"]) for p in values}),
    }

a_records = []
for p in packages:
    if p["cefrBand"] in ("A1", "A2"):
        path = p["_path"]
        relative = path.relative_to(ROOT).as_posix()
        a_records.append(f"{relative}|{sha(path)}|{path.stat().st_size}")
a12_record_hash = hashlib.sha256("\n".join(sorted(a_records)).encode()).hexdigest()

protected_paths = [
    "package.json", "package-lock.json", "src-tauri/Cargo.toml", "src-tauri/Cargo.lock",
    "local-ai/voice_coach_v2.py", "local-ai/voice_coach_v2_STABLE.py", "local-ai/voice_streaming_runtime.py",
    "local-ai/pronunciation/pronunciation_engine.py", "local-ai/pronunciation/pronunciation_core.py",
    "src-tauri/src/interactive_exercise.rs", "src-tauri/src/guided_conversation.rs",
    "src-tauri/src/interactive_lesson_analysis.rs",
    "src-tauri/prompts/conversation_teacher.txt", "src-tauri/prompts/lesson_analyzer_v1.txt",
    "src-tauri/resources/placement/placement_bank_v1.json", "src-tauri/src/placement_scoring.rs", "src-tauri/src/placement_evaluator.rs",
]
protected = {name: sha(ROOT / name) for name in protected_paths if (ROOT / name).is_file()}

uri = DB.as_uri() + "?mode=ro"
connection = sqlite3.connect(uri, uri=True)
tables = [row[0] for row in connection.execute("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")]
db = {
    "sha256": sha(DB),
    "schema": connection.execute("SELECT MAX(version) FROM schema_migration").fetchone()[0],
    "integrity": connection.execute("PRAGMA integrity_check").fetchone()[0],
    "foreignKeys": len(connection.execute("PRAGMA foreign_key_check").fetchall()),
    "counts": {table: connection.execute(f'SELECT COUNT(*) FROM "{table}"').fetchone()[0] for table in tables},
}
connection.close()

stage_fingerprints = Counter()
for package in packages:
    fingerprint = hashlib.sha256(json.dumps(package["stages"], sort_keys=True, ensure_ascii=False).encode()).hexdigest()
    stage_fingerprints[fingerprint] += 1

result = {
    "curriculum": {"version": curriculum["curriculumVersion"], "state": curriculum["publicationState"], "levels": len(curriculum["levels"]), "units": sum(len(level["units"]) for level in curriculum["levels"]), "lessons": len(refs), "order": [level["cefrLevel"] for level in curriculum["levels"]]},
    "packages": {"total": len(packages), "invalidJson": 0, "published": sum(p["publicationState"] == "published" for p in packages), "uniqueIds": len({p["lessonId"] for p in packages}), "allEightStages": all(len(p["stages"]) == 8 for p in packages), "brokenRefs": len(set(refs) - available), "orphanPackages": len(available - set(refs)), "duplicateWholeStagePayloads": sum(count - 1 for count in stage_fingerprints.values() if count > 1)},
    "metrics": metrics,
    "a1a2RecordHash": a12_record_hash,
    "a1a2MatchesBaseline": a12_record_hash == "022ec24db1155ebc1b140c9be63ee964060393d66f878aa5bf11b7874b67675b",
    "database": db,
    "protectedHashes": protected,
    "curriculumSha256": sha(CURRICULUM),
}

OUT.mkdir(parents=True, exist_ok=True)
(OUT / "FINAL_AUDIT.json").write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

final_roots = [LESSONS, CURRICULUM.parent, ROOT / "docs" / "content", ROOT / ".phase-y-artifacts", ROOT / ".phase-z-artifacts"]
final_files = [path for base in final_roots for path in base.rglob("*") if path.is_file()]
final_files.extend(ROOT / path for path in [
    "src-tauri/src/interactive_lesson_content.rs", "src-tauri/src/interactive_lesson_engine.rs", "src-tauri/src/curriculum.rs",
    "src/pages/CoursePage.tsx", "src/pages/CoursePage.test.tsx", "docs/CONTENT_EDITORIAL_STANDARD_V1.md",
    "package.json", "package-lock.json", "src-tauri/Cargo.toml", "src-tauri/Cargo.lock",
])
final_lines = [f"{sha(path)}  {path.relative_to(ROOT).as_posix()}" for path in sorted(final_files) if path.name != "FINAL_SHA256.txt"]
(OUT / "FINAL_SHA256.txt").write_text("\n".join(final_lines) + "\n", encoding="utf-8")

backup_files = [p for p in BACKUP.rglob("*") if p.is_file() and p.name not in ("BACKUP_SHA256.txt", "BACKUP_SHA256_FINAL.txt")]
backup_lines = [f"{sha(path)}  {path.relative_to(BACKUP).as_posix()}" for path in sorted(backup_files)]
(BACKUP / "BACKUP_SHA256_FINAL.txt").write_text("\n".join(backup_lines) + "\n", encoding="utf-8")
print(json.dumps(result, indent=2, ensure_ascii=False))
