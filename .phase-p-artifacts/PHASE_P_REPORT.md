# Fase P — Platform Hardening + Data Safety + Diagnostics

## A–K. Entrega, backups e versões

- **A. Arquivos criados:** `src/components/DataBackupSection.tsx`, teste correspondente, `src/hooks/useDataSafety.ts`, `src/pages/DiagnosticsPage.tsx`, teste correspondente, migration `013_platform_reliability.sql`, `reliability.rs`, `sha256.rs` e `system_diagnostics.rs`.
- **B. Arquivos modificados:** `src/App.tsx`, `src/App.test.tsx`, `src/components/AppLayout.tsx`, `src/pages/SettingsPage.tsx`, teste de Settings, `src/services/native.ts`, `src/types/index.ts`, `src-tauri/src/database.rs`, `lib.rs`, `paths.rs` e `pronunciation_engine.rs`.
- **C. Backup pré-Fase P:** `C:\ENGLISH AI COACH\.backup-phase-p\20260823-211641`.
- **D. Manifest SHA-256:** `manifest-sha256.txt`; hash do próprio manifest `C1BA1B23A027AE464F3E6A382045D2D7CDEC25C122AFDF557748383EB1E5D989`; 132 arquivos protegidos.
- **E. Database backup pré-migration:** `physical-db-before-013.sqlite3`, 438.272 bytes, SHA-256 `8D371D9E40FA4DBE101A86CC2C45CF77F8C89CEA3C47080E995A64BB07B8A24D`, criado pela SQLite Backup API com o banco ativo/WAL.
- **F. Migration 013:** criada.
- **G. Justificativa:** a tabela técnica limitada `app_system_event` viabiliza warnings/recovery sanitizados e auditáveis sem misturar conteúdo pedagógico ou logs livres.
- **H. Platform Reliability Schema Version:** 1.
- **I. Backup Format Version:** 1.
- **J. Diagnostic Report Version:** 1.
- **K. Startup Recovery Rule Version:** 1.

## L–AM. SQLite, backup e restore

- **L. SQLite/WAL:** migrations 001–012 preservadas; banco físico em `journal_mode=wal`, schema inicial 12 e final 13. Conexões são curtas, com busy timeout de 5 s e FK habilitada.
- **M–N. Snapshot consistente:** `VACUUM INTO` em conexão SQLite. Ele lê um snapshot transacional consistente, incorpora commits visíveis no WAL e produz banco independente. Teste manteve writer WAL aberto e confirmou o commit no backup.
- **O. Formato:** diretório único `*.eacbackup` contendo somente `database.sqlite3` e `manifest.json`; nenhuma dependência ZIP foi adicionada.
- **P. Diretório default:** `%LOCALAPPDATA%\com.englishaicoach.desktop\backups`.
- **Q. Manifest:** format version, criação, schema DB, app version real (`0.1.0`), nome do DB, SHA-256, settingsIncluded e allowlist de arquivo/tamanho/hash.
- **R. Incluído:** SQLite completo; settings já estão dentro dele.
- **S. Excluído:** modelos Qwen/Whisper/Piper/Wav2Vec2, venvs, áudio, temporários, caches, logs, `target`, `node_modules` e `dist`.
- **T–V. Validação:** SHA-256 próprio em Rust, `PRAGMA integrity_check=ok`, `PRAGMA foreign_key_check=0` e schema/required tables.
- **W–X. Publicação:** diretório `.partial` → validação → rename atômico; staging incompleto é removido e nunca recebe extensão válida.
- **Y. Arquitetura restore:** validação → safety backup obrigatório → `pending-restore.json` atômico → restart controlado → nova validação → staging → rename do DB atual para rollback → swap → migrations → integrity/FK → remoção do rollback.
- **Z. Compatibilidade:** backup format deve ser 1; schemas 1–13 aceitos; schema maior é rejeitado.
- **AA. Future schema:** rejeitado com “This backup was created by a newer version of English AI Coach.”
- **AB. Older schema:** restaurado em staging e migrado normalmente até 13; backup original não é alterado.
- **AC. Safety backup:** `pre-restore-safety-backup-*.eacbackup`; falha interrompe restore.
- **AD. Staging:** arquivo privado `restore-stage-<uuid>.sqlite3`.
- **AE. Connection lifecycle:** não há pool persistente; restore ocorre antes do AppState/repositórios abrirem conexões. Reopen acontece pelas conexões normais após startup.
- **AF. Windows locking:** swap somente no startup, antes das conexões; testado em Windows.
- **AG. WAL/SHM:** sidecars antigos são removidos antes do swap; validação immutable não cria sidecars no bundle.
- **AH. Rollback:** teste induziu falha pós-swap e confirmou recuperação exata do DB anterior.
- **AI. Refresh:** restart recarrega todas as páginas/caches e emite `english-ai-coach:data-restored`; backup emite `english-ai-coach:backup-created`.
- **AJ. Restart:** obrigatório e explicitado pela UI, escolhido por segurança de locking/lifecycle.
- **AK–AM. Segurança de path:** frontend envia somente `backupId`; backend aceita um único componente terminado em `.eacbackup`, rejeita traversal, symlink/reparse entry e qualquer arquivo fora da allowlist.

## AN–BP. Diagnostics, events, recovery e cleanup

- **AN. Diagnostics:** novo compositor Rust sobre Local AI Probe + verificações read-only de DB/settings/arquivos; nenhuma mutação pedagógica.
- **AO. Componentes:** Database, Ollama, Whisper, Piper, Voice Bridge, Voice Streaming, Pronunciation e Settings.
- **AP. Database:** schema, WAL, integrity e FK.
- **AQ–AR. Ollama/Qwen:** somente `GET /api/tags`; endpoint e presença exata de `qwen3.5:4b`.
- **AS. Whisper:** `whisper-cli.exe`, `ggml-small.en-q5_1.bin` e 12 threads.
- **AT. Piper:** Python/`piper-tts`, versão, `en_US-lessac-medium.onnx` e JSON.
- **AU–AV. Voice:** bridge e `voice_streaming_runtime.py`; runtime version 1.
- **AW–AX. Pronunciation:** venv, worker, arquivos essenciais, manifest, model id/revision e metadata. Health leve não re-hasheia 1,2 GB; full hashes permanecem no manifest instalado para validação sob demanda futura.
- **AY. Status:** `healthy`, `warning`, `unavailable`.
- **AZ. Readiness:** `databaseReady`, `conversationReady` e `pronunciationReady` independentes; sem score global enganoso.
- **BA. Zero geração:** confirmado fisicamente `generationInvoked=false`; diagnostics não referencia/invoca `/api/chat`.
- **BB. Report:** JSON versionado, typed DTO, timestamp, app/platform e componentes.
- **BC. Sanitização:** sem transcript, profile, vocabulary, target, prompt, áudio ou conteúdo de arquivo; teste com fixtures-secret.
- **BD. System events:** migration 013, schema version 1, severity restrita, component/code, details JSON allowlisted e timestamps.
- **BE. Retention:** últimos 300; teste inseriu 305 e confirmou 300.
- **BF. Startup recovery:** paths → pending restore → migration → stale lesson/analysis recovery → cleanup próprio → AppState.
- **BG. Lesson:** `starting/active` stale → `interrupted`, nunca completed.
- **BH. Review:** `in_progress` permanece resumível; nenhuma recuperação automática.
- **BI. Placement:** `in_progress` permanece resumível; nenhuma recuperação automática.
- **BJ. Pronunciation:** schema v1 só persiste resultados terminais; não existe row `analyzing` para ficar stale. A atividade runtime agora é rastreada para bloquear restore.
- **BK. Voice performance:** métricas parciais permanecem intactas; nenhuma fabricação/reparo.
- **BL–BM. Cleanup:** somente diretório privado `temporary_audio` e nomes `placement-*`, `pronunciation-*`, `voice-n-*` ou UUID com sufixos internos conhecidos.
- **BN. WAV do usuário:** WAV desconhecido, fora do diretório privado, modelo e backup foram preservados em testes.
- **BO–BP. Processos:** Job Objects existentes mantidos; restore encerra apenas worker de Pronunciation gerenciado/idle. Nenhum `taskkill` genérico, kill por nome ou busca de `python.exe` foi adicionado.

## BQ–CC. UI, navegação e comandos

- **BQ. Settings:** seção Data & Backup com Create, Restore, Last/available backups, Open Folder e privacy label.
- **BR. Backup states:** idle, creating, validating, completed, failed; double-click/mutual exclusion no backend e UI.
- **BS. Restore states:** validating, creating safety backup, restoring, completed/failed e pending restart.
- **BT. Confirmação:** texto explícito sobre substituição e safety backup; Cancel recebe foco.
- **BU–BX. Diagnostics page:** `/diagnostics`, cards independentes, readiness, advanced details, recent events, retry e Copy Diagnostic Report.
- **BY. Navegação:** Diagnostics na sidebar e link em Settings.
- **BZ. Responsividade:** sidebar ganhou scroll vertical; rotas resetam scroll; main foi limitado à largura disponível; toggles empilham até `xl` para escala alta.
- **CA. Tauri commands:** `get_system_diagnostics`, `export_diagnostic_report`, `create_app_backup`, `list_app_backups`, `validate_app_backup`, `restore_app_backup`, `get_backup_status`, `get_backup_directory`, `open_backup_folder`, `list_recent_system_events`.
- **CB. Rust:** reliability, SHA-256 e diagnostics; database/paths/startup integrados; repositórios pedagógicos reutilizados sem duplicação.
- **CC. Frontend:** DTOs sem `any`, serviços typed, hook `useDataSafety`, DataBackupSection e DiagnosticsPage.

## CD–DN. Testes funcionais e de segurança

- **CD–CH:** manifest válido, missing manifest, bad version, missing DB e SHA mismatch: passaram.
- **CI:** WAL consistency com writer aberto/autocheckpoint desabilitado: passou.
- **CJ:** dois backups independentes/idempotentes: passou.
- **CK:** restore current-version round-trip exato: passou.
- **CL:** schema 1 → restore → migrations → schema 13: passou.
- **CM:** future schema 99 rejeitado: passou.
- **CN:** DB corrompido/bad hash rejeitado antes do swap: passou.
- **CO:** FK violation rejeitada: passou.
- **CP:** rollback pós-swap induzido: passou.
- **CQ:** safety backup existe antes do pending/swap: passou.
- **CR–CS:** invariantes existentes de Review resume e Placement resume/start-over continuam passando.
- **CT:** Pronunciation não possui estado persistido transitório; repository só grava terminal e teste confirma ausência de fabricação.
- **CU:** stale Lesson → interrupted: passou.
- **CV:** recovery é idempotente por update restrito a estados transitórios; migrations também idempotentes.
- **CW–CZ:** cleanup permitido, WAV desconhecido, modelo e backup retention: passaram.
- **DA:** DB diagnostics unitário + físico healthy schema 13: passou.
- **DB:** Local AI/Ollama required-component tests + probe físico: passou.
- **DC:** diagnóstico usa apenas tags; relatório físico confirmou `generationInvoked=false`.
- **DD–DE:** Whisper/Piper checks físicos healthy.
- **DF–DG:** Pronunciation manifest/runtime healthy e independência de conversation readiness testada na UI.
- **DH:** report sanitization: passou.
- **DI:** retention 300: passou.
- **DJ:** loading, healthy, optional unavailable, DB warning, rerun, advanced e copy: passaram.
- **DK:** idle/create/completed/failed/last backup: passaram.
- **DL:** confirmação, progress/schedule/success/error: passaram; future/corrupt são decisões Rust.
- **DM:** restore disabled com blocker runtime: passou na UI; Voice/Pronunciation decididos pelo Rust.
- **DN:** botões sem mouse obrigatório, roles/ARIA/focus e navegação por teclado verificados; nenhuma `dangerouslySetInnerHTML`.

## DO–EZ. Qualidade, físico, preservação e dívidas

- **DO. Typecheck:** passou.
- **DP. Lint:** passou.
- **DQ. Frontend:** 28 arquivos, 117 testes, todos passaram.
- **DR. Rust:** 159 descobertos; 145 passaram; 14 testes físicos/manuais ignorados por padrão; zero falhas. `cargo fmt --check` e `cargo check --offline` passaram.
- **DS. Python:** nenhum Python modificado.
- **DT. Streaming:** 15/15 passaram no venv local correto. Uma tentativa inicial com Python global falhou por ausência de NumPy; nenhum pacote foi instalado.
- **DU. Pronunciation:** 12/12 passaram.
- **DV. Vite:** passou; 1.850 módulos; JS ~397,49 KB, CSS ~37,42 KB.
- **DW. Tauri:** debug `--no-bundle` passou; nenhum installer.
- **DX–ED. Backup humano:** criado e validado.
- **DY. Path:** `%LOCALAPPDATA%\com.englishaicoach.desktop\backups\EnglishAICoach-Physical-Backup-1787532012-6ef07d7f.eacbackup`.
- **DZ. Size:** 450.560 bytes.
- **EA. SHA-256:** `D38B82FF49AE0A8AE63B3A974CEB3123D9B065A8AA4FE95AF97B023A1E178741`.
- **EB. Schema:** 13.
- **EC. Integrity:** ok.
- **ED. FK:** zero.
- **EE. Restore físico:** validado somente em cópia temporária; schema 13, integrity ok, FK zero.
- **EF. Banco humano:** não foi restaurado destrutivamente.
- **EG–EM. Diagnostics físico:** Database, Ollama/Qwen, Whisper, Piper 1.7.0, Voice Bridge, Streaming v1 e Pronunciation Wav2Vec2: todos healthy; overall “All systems ready”.
- **EN. Temp audit:** zero arquivos residuais após startup cleanup.
- **EO–EP. UI:** startup, sidebar/links, Settings e escala Windows inspecionados em 1295×737; os overflows encontrados foram corrigidos. Capturas estão em `.phase-p-artifacts`.
- **EQ. Counts:** todos os counts pedagógicos antes/depois idênticos. Apenas `schema_migration` 12→13 e nova `app_system_event=0`.
- **ER–EU. Preservação:** gamification (3 unlocks/5 XP events), CEFR/Placement (4 attempts/27 answers), Review (0/0), Pronunciation (1 attempt/3 words) inalterados.
- **EV–EW:** SQLite integrity ok; FK zero.
- **EX:** implementação e testes offline; nenhum cloud/telemetry.
- **EY. Problemas:** cópia inicial do backup de código omitiu diretórios por wildcard literal; detectada antes de edits, corrigida para 132 arquivos e manifest regenerado. Python global não tinha NumPy; Streaming foi repetido no venv correto. A inspeção visual encontrou sidebar/route/toggle overflow e gerou três correções responsivas.
- **EZ. Dívidas futuras:** full re-hash on-demand do modelo de Pronunciation; backup automático/scheduler; picker de backup externo/ZIP somente se uma dependência já autorizada for adotada; tela especial de recuperação para DB que não abre antes do AppState; instrumentação adicional de crash event diretamente dos workers.

## FA. Confirmações finais

- `voice_coach_v2.py` e `voice_coach_v2_STABLE.py` intactos: SHA-256 `F56E16A71130C0BC4974DF13038D5937DA611AF3C90B2CE4C7891F28523D2E2D`.
- Voice Streaming Runtime v1 intacto: `8A8BB8FB0CFAB51F37BABC6839FF012C8C051483DA7C57AA251C08CB79E2EAFE`.
- Conversation prompt intacto: `8B5E07911A50F18E23C6338F8521660BF4CEC652496C785F4B40A4B57056F19D`.
- Lesson Analyzer prompt/schema, Placement scoring/bank/evaluator, Student Profile, Learning Memory, Gamification XP v1 e Review queue semantics intactos.
- Pronunciation Engine/Score Version 1 intactos; normal Lesson pronunciation continua null; modelo acústico/revision intactos.
- Whisper continua `ggml-small.en-q5_1.bin`, 12 threads; Conversation VAD 3,5 s; Qwen `qwen3.5:4b` think false; Piper `en_US-lessac-medium`.
- Zero chamada LLM adicional; diagnostics não geram conversa; nenhum dado pedagógico enviado externamente.
- Backup é local e exclui modelos, venvs e áudio.
- Restore sempre valida e cria safety backup; nenhum restore automático/destrutivo foi executado no banco humano.
- Nenhum kill genérico; arquivos fora do temp privado nunca são removidos.
- Sem cloud backup, telemetry, download de modelo, pacote/crate/plugin novo, `setup-windows.ps1`, `ollama pull`, Git, installer ou auto-update.
- Interactive Lessons, Theory, Visual Vocabulary, Listening Lesson, Interactive Repeat, Speaking Gate, Exercise Engine, Guided Interactive Lesson, Curriculum e PDF inteligente não foram implementados.
- Próxima fase não foi iniciada.
