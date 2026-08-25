# Fase S — Auditoria pré-implementação

Data da auditoria: 2026-08-24  
Projeto oficial: `C:\ENGLISH AI COACH`  
Escopo auditado: Listening v1, Repeat v1 e Speaking Check v1 sobre a fundação da Fase R.

## Resultado executivo

A base da Fase R é adequada para extensão, mas a Fase S não pode ser implementada apenas preenchendo `completion_json`. São necessárias uma migration 015, uma camada de runtime de áudio guiado independente da conversa, progresso tipado por item, tentativas guiadas próprias e integração controlada com o Pronunciation Engine existente.

Não será usado Qwen/Ollama. Não serão instalados pacotes, crates, modelos ou plugins. O fluxo livre de voz, streaming, prompts, placement, analyzer, gamificação, review e o núcleo acústico permanecerão protegidos.

## A. Como o Piper é chamado hoje

Há dois caminhos existentes:

1. `src-tauri/src/speech.rs::synthesize` procura `piper.exe` em `tools/piper` ou no `PATH`, procura a voz em `voices/`, executa um subprocesso por fala, devolve WAV em base64 e apaga o arquivo temporário.
2. O runtime Voice V2 usa `local-ai/piper/.venv/Scripts/python.exe`, carrega `PiperVoice` persistentemente e possui fallback para `python -m piper -m <modelo> -f <wav> -- <texto>`. Ele adiciona 0,5 s de silêncio somente ao primeiro chunk.

A instalação oficial disponível nesta máquina está em `local-ai/piper`: Python, modelo ONNX e JSON existem. Portanto, o TTS legado de `speech.rs` não é um caminho confiável para a Fase S e o runtime protegido de conversa não deve ser acoplado a aulas guiadas.

## B. API de TTS reutilizável

Não existe uma API atual que, ao mesmo tempo, use a instalação oficial do Piper, tenha identidade de reprodução verificável, cache por item, cancelamento isolado, regra de uma reprodução guiada por vez e confirmação backend de término.

Decisão: criar `GuidedLessonAudioRuntime` versão 1, isolado do Voice V2. Ele reutilizará o Piper local aprovado por subprocesso, sem iniciar Whisper, Wav2Vec2, Ollama ou o bridge de conversa. Áudio gerado terá prefixo privado `guided-lesson-`; a resposta terá `playbackId`, e a conclusão só será aceita para a reprodução ativa e após a duração mínima do WAV.

## C. Como Pronunciation Practice grava hoje

`usePronunciationPractice.ts` controla diretamente `AudioContext`, `getUserMedia`, o `AudioWorklet` `/pcm-capture-processor.js`, trim, downsample e WAV. A captura final é PCM mono, 16-bit, 16 kHz e tem limite de 15 segundos. O backend grava um WAV temporário `pronunciation-*`, executa o content check pelo Whisper e a análise acústica pelo worker persistente Wav2Vec2, remove o WAV e persiste apenas o resultado estruturado.

## D. Existe hook genérico de gravação?

Não. O processador PCM é reutilizável, mas os hooks de Pronunciation e Placement estão acoplados aos respectivos fluxos. A Fase S criará um hook de gravação guiada compartilhado por Repeat e Speaking Check, usando o mesmo AudioWorklet e o mesmo contrato PCM/WAV. Não usará o VAD de 3,5 s da conversa.

## E. `source_type` atual de pronúncia

O repositório aceita somente `custom`, `vocabulary` e `diagnostic`. O comando público aceita somente `custom` e `vocabulary`. Tentativas guiadas não podem ser mascaradas como tentativas avulsas.

Decisão: usar `source_type = 'interactive_lesson'`; `source_id` apontará para a tentativa guiada correspondente. A listagem recente da página Pronunciation será filtrada para fontes avulsas e não exibirá tentativas guiadas.

## F. `CHECK` de `source_type` no banco

A migration 012 contém um `CHECK` fechado para `custom`, `vocabulary` e `diagnostic`. SQLite não permite alterar esse `CHECK` diretamente.

Decisão: a migration 015 reconstruirá com segurança `pronunciation_attempt` e `pronunciation_word_result`, preservando linhas e chaves, mantendo a restrição e acrescentando explicitamente `interactive_lesson`. Os índices serão recriados e a migração será coberta por upgrade 14→15, banco novo 1→15, idempotência, integrity check e foreign-key check.

## G. Progresso parcial por item

`interactive_lesson_stage_state` registra somente estado da etapa, `attempt_count` 0/1 e um JSON final. Ele não registra reproduções concluídas por segmento, referência ouvida, tentativas repetidas nem seleção da tentativa exibida.

Decisão: adicionar estado de runtime tipado por etapa e tabela de tentativas guiadas. Listening guardará contagens concluídas por segmento. Repeat guardará referência concluída e tentativa selecionada por alvo. Speaking Check guardará tentativa selecionada por alvo. O backend construirá e validará esses estados; a UI não enviará JSON arbitrário.

## H. `completion_json` é suficiente?

Não. Ele é adequado como snapshot final resumido, mas insuficiente para retomada após fechamento, progresso parcial, retry, seleção explícita e recuperação de análise interrompida. Continuará sendo usado somente para o resultado terminal (`listening_completed`, `repeat_completed` ou `speaking_check_completed`) sem áudio, transcript ou caminhos.

## I. Por que a migration 015 é necessária

A migration `015_interactive_lesson_audio_practice.sql` é necessária para:

- persistir progresso parcial e retomável dos três novos tipos;
- persistir tentativas guiadas separadamente, inclusive falhas/cancelamentos e retry;
- preservar a seleção atual, sem escolher automaticamente a melhor nota;
- recuperar tentativas `analyzing` interrompidas sem falhar a sessão;
- registrar a proveniência acústica real `interactive_lesson` no banco;
- manter áudio e transcript fora do banco;
- permitir auditoria e integridade relacional sem sobrecarregar o estado terminal da Fase R.

## Contratos e invariantes confirmados

- O package schema permanece versão 1 e o lesson flow permanece versão 1.
- `Theory` e `VisualVocabulary` continuam disponíveis; `Listening`, `Repeat` e `SpeakingCheck` serão habilitados. `Exercise`, `GuidedConversation` e `Analysis` continuarão indisponíveis.
- Listening terá 1–12 segmentos, texto de até 240 caracteres, asset opcional e `revealTextAfterFirstPlay`.
- Repeat reutilizará exatamente os limites de Pronunciation: texto de 1–160 caracteres e 1–12 palavras.
- Speaking Check não terá áudio de referência nem autoplay.
- Somente `status = completed` pode ser selecionado e avançar. Não haverá nota mínima: inclusive score 12 ou 0 e baixa confidence podem prosseguir.
- `content_mismatch`, `insufficient_audio`, `alignment_failed`, `engine_unavailable`, `cancelled` e `failed` nunca satisfazem conclusão.
- Play iniciado, parcial, cancelado ou com erro não incrementa progresso. Replay concluído incrementa a contagem.
- Retry cria nova tentativa. Continue seleciona a tentativa concluída atualmente exibida, nunca a melhor automaticamente.
- Nenhum áudio ou transcript será persistido. Assets de pacote serão copiados de forma imutável e privada por sessão para permitir retomada mesmo se a origem desaparecer; WAV gerado terá cache temporário de runtime e limpeza por propriedade explícita.
- Acordar Bluetooth adicionará 0,5 s ao início de cada reprodução guiada, sem reescrever o asset original.
- A reprodução exige ação do usuário e será única por runtime guiado. Eventos stale serão rejeitados por `playbackId`, sessão, etapa e item.
- Gravar interromperá reprodução; não será permitido reproduzir enquanto grava. O limite rígido será 15 s e não haverá VAD de conversa.

## Baseline protegido

Hashes SHA-256 registrados antes da implementação:

- `local-ai/voice_coach_v2.py`: `F56E16A71130C0BC4974DF13038D5937DA611AF3C90B2CE4C7891F28523D2E2D`
- `local-ai/voice_streaming_runtime.py`: `8A8BB8FB0CFAB51F37BABC6839FF012C8C051483DA7C57AA251C08CB79E2EAFE`
- `local-ai/pronunciation/pronunciation_engine.py`: `FA28B35D8948D325AF79686276EB51703677A2187B31215D06EF669147BE0968`

Esses arquivos não serão alterados pela Fase S. Os modelos existentes também não serão modificados.

## Ordem segura de execução

1. Criar backup versionado de todos os arquivos existentes que serão alterados e manifest SHA-256.
2. Criar backup SQLite consistente `physical-db-before-015.sqlite3` e validar integridade.
3. Implementar migration 015 e contratos tipados.
4. Implementar runtime de áudio, progresso, tentativas e integração acústica.
5. Implementar UI acessível e gravação compartilhada de Repeat/Speaking Check.
6. Atualizar documentação/schema/fixtures isoladas e testes.
7. Executar regressões completas, build Tauri debug sem bundle, migração física e relatório final A–IP.
