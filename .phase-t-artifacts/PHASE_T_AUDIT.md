# Fase T — auditoria pré-implementação

Data: 2026-08-24  
Projeto oficial: `C:\ENGLISH AI COACH`  
Estado auditado: schema 15 / Fase S concluída.

## Escopo inspecionado

- migrations 014 e 015;
- package schema v1 e authoring guide;
- `InteractiveLessonContentRegistry`, capability registry e validação Rust;
- modelos tipados, stage payloads, executors e session engine;
- `interactive_lesson_stage_state`, runtime state e `completion_json`;
- snapshot imutável do package e snapshot de assets;
- comandos Tauri e construção dos DTOs públicos;
- runner React, fluxo itemizado da Fase S e componentes compartilhados da Fase Q;
- backup/restore, schema version, startup recovery, system events e serialização JSON.

## Respostas obrigatórias

### A. Existe tabela genérica de stage attempts?

Não. A migration 014 contém sessão e estado por stage. A migration 015 acrescenta runtime state genérico e uma tabela específica de tentativas de pronúncia. Não existe uma tabela genérica adequada para tentativas determinísticas de Exercise.

### B. Ela suporta response/result JSON tipado?

Não se aplica a uma tabela genérica inexistente. A tabela de pronúncia possui `result_json`, mas sua semântica, estados e foreign keys são acústicos. Reutilizá-la para Exercise seria incorreto. O runtime state aceita JSON versionado, porém deve guardar somente progresso derivado e não um histórico potencialmente grande de tentativas.

### C. Ela é semanticamente adequada para Exercise?

Não. Exercise precisa de resposta imutável, resultado determinístico imutável, índice monotônico por item e seleção explícita única. Isso requer persistência relacional própria. O runtime state existente é adequado apenas para `currentExerciseIndex`, contadores e `selectedAttemptId`.

### D. Precisamos de migration 016?

Sim. Criar `interactive_lesson_exercise_attempt`, com resposta/resultados JSON versionados, `correct`, timestamps, unicidade por sessão/stage/exercise/índice e índice parcial garantindo no máximo uma tentativa selecionada por item. As migrations 001–015 permanecerão intocadas.

### E. Como stage runtime state está versionado?

`interactive_lesson_stage_runtime_state` usa chave composta `(session_id, stage_id)`, `runtime_state_schema_version = 1` e `state_json`. O enum Rust `GuidedStageRuntimeState`, serializado com tag `kind` em snake_case e campos camelCase, é a fonte tipada. Exercise acrescentará uma variante `exercise` schema v1, construída exclusivamente pelo backend.

### F. Como stage snapshots são recuperados?

No início da sessão, o package tipado completo é serializado em `interactive_lesson_session.package_snapshot_json`, junto do hash e das versões. Toda leitura de sessão e toda ação de stage desserializa esse snapshot. Assets de áudio declarados são copiados para o diretório privado da sessão. Exercise será corrigido exclusivamente contra o `package_snapshot_json`; o source content registry e o arquivo `lesson.json` não serão lidos durante grading.

### G. Como o backend evita expor private stage data?

Hoje os payloads privados são convertidos manualmente em `ActiveStageContentDto` dentro de `read_session`; tipos sem executor retornam `None`. Para Exercise será criada uma DTO pública tipada diferente do payload do package. Essa conversão copiará prompt, instruções, hint e opções/tokens/itens públicos, mas nunca `correctOptionId`, `correctOptionIds`, `acceptedAnswers`, `correctOrder`, `correctPairs` ou feedback pós-submit. A resposta esperada só poderá sair no DTO de uma tentativa realmente persistida após submissão.

### H. Como os frontend DTOs são construídos?

Rust é a fronteira de autoridade. Structs/enums Serde usam `camelCase` externamente e tags discriminadas `kind`/`exerciseType`; `deny_unknown_fields` protege package e requests. `src/types/index.ts` espelha somente os DTOs públicos e `src/services/native.ts` encapsula os comandos Tauri. O frontend nunca recebe o package snapshot bruto.

## Decisões de implementação

- manter Package Schema Version 1 e Flow Version 1;
- ativar somente Exercise schema v1; Guided Conversation e Analysis continuam fechados;
- criar `INTERACTIVE_EXERCISE_ENGINE_VERSION = 1`, `EXERCISE_STAGE_SCHEMA_VERSION = 1`, `EXERCISE_ATTEMPT_RESULT_VERSION = 1` e `EXERCISE_NORMALIZATION_VERSION = 1`;
- implementar os seis tipos por enums tipados Rust, sem `serde_json::Value` como domínio principal;
- usar NFKC, whitespace determinístico, apóstrofos/aspas tipográficas, lowercase e remoção de no máximo um sinal terminal `.?!`; preservar pontuação interna;
- não usar LLM, Whisper, Piper, Wav2Vec2, rede, áudio, transcript ou qualquer side effect pedagógico/global;
- `Continue` seleciona exatamente a tentativa exibida; nenhuma tentativa “melhor” é escolhida automaticamente;
- conclusão exige uma tentativa selecionada por item, nunca correção mínima; 0% pode concluir;
- transição de stage e `completion_json` permanecem na transação oficial do engine;
- fixtures ficam somente em `src-tauri/test-fixtures/interactive-lessons-phase-t` e usam TEMP DB/content root.

## Estado antes da Fase T

READY: Theory, Visual Vocabulary, Listening, Repeat, Speaking Check.  
NOT READY: Exercise, Guided Conversation, Analysis.

Nenhum arquivo preexistente foi modificado durante esta auditoria.
