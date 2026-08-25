export type ConversationState =
  | "IDLE"
  | "PREPARING"
  | "LISTENING"
  | "STUDENT_SPEAKING"
  | "TRANSCRIBING"
  | "TEACHER_THINKING"
  | "TEACHER_SPEAKING"
  | "TEACHER_CANCELLING"
  | "PAUSED"
  | "ENDING"
  | "ANALYZING"
  | "COMPLETED"
  | "ERROR";

export type ComponentName =
  | "ollama"
  | "llm"
  | "whisper"
  | "vad"
  | "piper"
  | "microphone"
  | "speaker"
  | "sqlite";

export interface ComponentHealth {
  name: ComponentName;
  label: string;
  ready: boolean;
  detail: string;
  repairHint?: string;
}

export interface Diagnostics {
  components: ComponentHealth[];
  dataDirectory: string;
  offlineReady: boolean;
  platform: string;
}

export interface OptionalLocalModelProbe {
  name: string;
  found: boolean;
  path: string;
}

export interface LocalVoiceEngineProbe {
  projectRoot: string;
  localAiRoot: string;
  whisper: {
    cliFound: boolean;
    cliPath: string;
    streamFound: boolean;
    streamPath: string;
    modelFound: boolean;
    modelPath: string;
    modelName: string;
    threads: number;
    additionalModels: OptionalLocalModelProbe[];
  };
  ollama: {
    reachable: boolean;
    baseUrl: string;
    modelFound: boolean;
    modelName: string;
  };
  piper: {
    pythonFound: boolean;
    pythonPath: string;
    installed: boolean;
    version: string | null;
    voiceFound: boolean;
    voiceConfigFound: boolean;
    voiceModelPath: string;
    voiceConfigPath: string;
    voiceName: string;
  };
  voiceDefaults: {
    whisperModel: string;
    whisperThreads: number;
    silenceToStopSeconds: number;
    preRollSeconds: number;
    startVoiceBlocks: number;
    minimumVoiceThreshold: number;
    noiseMultiplier: number;
    piperVoice: string;
    ttsStartSilenceSeconds: number;
    ollamaModel: string;
    ollamaThinking: boolean;
  };
  optionalComponents: {
    sileroFound: boolean;
    sileroPath: string;
  };
  offlineReady: boolean;
  problems: string[];
}

export type VoiceEngineState =
  "stopped" | "starting" | "running" | "stopping" | "error";

export interface VoiceEngineStatus {
  state: VoiceEngineState;
  processId: number | null;
}

export type LessonStatus =
  "starting" | "active" | "completed" | "interrupted" | "failed";

export interface Lesson {
  id: string;
  startedAt: string;
  endedAt: string | null;
  status: LessonStatus;
  topic: string | null;
  mode: string;
  durationSeconds: number | null;
  studentTurnCount: number;
  teacherTurnCount: number;
  correctionCount: number;
  whisperModel: string;
  whisperThreads: number;
  ollamaModel: string;
  piperVoice: string;
  voiceEngineVersion: string;
  errorMessage: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface TranscriptMessage {
  id: string;
  lessonId: string;
  sequenceIndex: number;
  turnIndex: number;
  role: "student" | "teacher";
  text: string;
  source: string;
  engineEventType: string;
  createdAt: string;
}

export interface CorrectionCandidate {
  id: string;
  lessonId: string;
  studentMessageId: string;
  teacherMessageId: string;
  studentText: string;
  teacherResponseText: string;
  detectionMethod: string;
  createdAt: string;
}

export interface LessonSummary {
  lessonId: string;
  status: LessonStatus;
  startedAt: string;
  endedAt: string | null;
  durationSeconds: number | null;
  studentTurns: number;
  teacherTurns: number;
  correctionCandidates: number;
}

export interface StartLessonResult {
  lessonId: string;
  lessonStatus: LessonStatus;
  voiceEngineState: VoiceEngineState;
  lesson: Lesson;
  memorySnapshot: TeacherMemorySnapshot;
  profileSnapshot: LessonStudentProfileSnapshot;
  configuration: LessonConfiguration;
}

export type LessonModeId =
  | "free_conversation"
  | "everyday_english"
  | "travel_english"
  | "job_interview"
  | "university_academic"
  | "debate_opinions"
  | "custom";
export type LessonDifficulty = "easy" | "standard" | "challenging";
export type LessonFocusArea =
  | "grammar"
  | "vocabulary"
  | "fluency"
  | "naturalness"
  | "verb_tenses"
  | "prepositions"
  | "interview_answers"
  | "academic_explanations"
  | "opinion_building";

export interface LessonModeDefinition {
  id: LessonModeId;
  version: number;
  title: string;
  description: string;
  defaultDifficulty: LessonDifficulty;
  supportedDifficulties: LessonDifficulty[];
  availableFocusAreas: LessonFocusArea[];
  allowsTopic: boolean;
  allowsObjective: boolean;
  allowsScenario: boolean;
  allowsCustomTitle: boolean;
}

export interface LessonStartRequest {
  modeId: LessonModeId;
  difficulty: LessonDifficulty;
  topic?: string;
  objective?: string;
  scenario?: string;
  focusAreas: LessonFocusArea[];
  customTitle?: string;
}

export interface LessonConfiguration {
  lessonId: string;
  modeId: LessonModeId;
  modeVersion: number;
  modeTitle: string;
  lessonModeContextVersion: number;
  difficulty: LessonDifficulty;
  topic: string | null;
  objective: string | null;
  scenario: string | null;
  focusAreas: LessonFocusArea[];
  customTitle: string | null;
  configurationSchemaVersion: number;
  createdAt: string;
  legacy: boolean;
}

export interface TeacherMemorySnapshot {
  enabled: boolean;
  contextLoaded: boolean;
  contextVersion: number | null;
  summarySchemaVersion: number;
  analyzedLessonCountUsed: number;
}

export type LessonAnalysisStatus =
  "pending" | "running" | "completed" | "failed" | "insufficient_data";

export interface LessonAnalysisScores {
  fluency: number;
  grammar: number;
  vocabulary: number;
  comprehension: number;
  interaction: number;
  pronunciation: null;
}

export interface LessonAnalysisStrength {
  title: string;
  evidence: string;
}

export interface LessonAnalysisImprovement {
  area: string;
  title: string;
  explanation: string;
  exampleFromLesson: string;
  betterAlternative: string;
}

export type LessonAnalysisCorrectionCategory =
  | "grammar"
  | "vocabulary"
  | "word_choice"
  | "verb_tense"
  | "preposition"
  | "article"
  | "word_order"
  | "naturalness"
  | "other";

export interface LessonAnalysisCorrection {
  original: string;
  corrected: string;
  explanation: string;
  category: LessonAnalysisCorrectionCategory;
}

export interface LessonAnalysisNaturalAlternative {
  original: string;
  alternative: string;
}

export interface LessonAnalysisVocabulary {
  wordOrPhrase: string;
  meaning: string;
  example: string;
}

export interface LessonAnalysisRecurringPattern {
  pattern: string;
  count: number;
  explanation: string;
}

export interface LessonAnalysis {
  id: string;
  lessonId: string;
  status: LessonAnalysisStatus;
  schemaVersion: number;
  promptVersion: number;
  analyzerModel: string;
  startedAt: string | null;
  completedAt: string | null;
  overallScore: number | null;
  scores: LessonAnalysisScores | null;
  strengths: LessonAnalysisStrength[];
  priorityImprovements: LessonAnalysisImprovement[];
  corrections: LessonAnalysisCorrection[];
  naturalAlternatives: LessonAnalysisNaturalAlternative[];
  vocabulary: LessonAnalysisVocabulary[];
  recurringPatterns: LessonAnalysisRecurringPattern[];
  nextLessonRecommendations: string[];
  summary: string | null;
  pronunciationAvailable: boolean;
  errorMessage: string | null;
  createdAt: string;
  updatedAt: string;
}

export type LessonHistoryFilter =
  "all" | "completed" | "interrupted" | "analyzed" | "unanalyzed";

export interface LessonHistoryItem {
  id: string;
  startedAt: string;
  endedAt: string | null;
  durationSeconds: number | null;
  status: LessonStatus;
  topic: string | null;
  mode: string;
  modeId: LessonModeId;
  modeTitle: string;
  customTitle: string | null;
  studentTurnCount: number;
  teacherTurnCount: number;
  correctionCount: number;
  analysisStatus: LessonAnalysisStatus | null;
  overallScore: number | null;
}

export interface LessonHistoryPage {
  items: LessonHistoryItem[];
  total: number;
  limit: number;
  offset: number;
}

export interface DashboardLatestAnalysis {
  lessonId: string;
  startedAt: string;
  durationSeconds: number | null;
  overallScore: number;
  scores: LessonAnalysisScores;
}

export interface DashboardSummary {
  totalLessons: number;
  completedLessons: number;
  totalPracticeSeconds: number | null;
  totalStudentTurns: number;
  totalCorrections: number;
  analyzedLessons: number;
  averageOverallScore: number | null;
  latestLesson: LessonHistoryItem | null;
  latestAnalyzedLesson: DashboardLatestAnalysis | null;
  latestRecommendation: string | null;
}

export type ScoreDimension =
  "fluency" | "grammar" | "vocabulary" | "comprehension" | "interaction";

export interface ProgressAverages {
  overall: number;
  fluency: number;
  grammar: number;
  vocabulary: number;
  comprehension: number;
  interaction: number;
}

export interface ProgressPoint {
  lessonId: string;
  date: string;
  durationSeconds: number | null;
  overall: number;
  fluency: number;
  grammar: number;
  vocabulary: number;
  comprehension: number;
  interaction: number;
}

export interface ProgressOverview {
  analyzedLessonCount: number;
  averages: ProgressAverages | null;
  strongestAreas: ScoreDimension[];
  focusAreas: ScoreDimension[];
  points: ProgressPoint[];
  latestRecommendation: string | null;
}

export interface LessonDetails {
  lesson: Lesson;
  configuration: LessonConfiguration;
  messages: TranscriptMessage[];
  correctionCandidates: CorrectionCandidate[];
  analysis: LessonAnalysis | null;
  studentProfileSnapshot: LessonStudentProfileSnapshot | null;
}

export type VocabularyStatus = "new" | "learning" | "known";
export type VocabularyFilter = "all" | VocabularyStatus;
export type VocabularySort =
  "recently_seen" | "first_seen" | "most_frequent" | "alphabetical";

export interface VocabularySummary {
  total: number;
  new: number;
  learning: number;
  known: number;
  contributingLessons: number;
}

export interface LearningMemorySummary {
  vocabularyTotal: number;
  vocabularyNew: number;
  vocabularyLearning: number;
  vocabularyKnown: number;
  lessonsContributingVocabulary: number;
  recurringMistakesConfirmed: number;
}

export interface VocabularyItem {
  id: string;
  text: string;
  meaning: string;
  status: VocabularyStatus;
  firstSeenAt: string;
  lastSeenAt: string;
  lessonCount: number;
  occurrenceCount: number;
  latestExample: string | null;
}

export interface VocabularyPage {
  items: VocabularyItem[];
  total: number;
  limit: number;
  offset: number;
}

export interface VocabularyOccurrence {
  lessonId: string;
  lessonDate: string;
  example: string;
  occurrenceCount: number;
}

export interface VocabularyItemDetails {
  item: VocabularyItem;
  occurrences: VocabularyOccurrence[];
}

export interface RecurringMistake {
  id: string;
  category: LessonAnalysisCorrectionCategory;
  title: string;
  explanation: string;
  lessonCount: number;
  occurrenceCount: number;
  firstSeenAt: string;
  lastSeenAt: string;
  status: "active" | "improving" | "resolved";
}

export interface MistakeOccurrence {
  lessonId: string;
  lessonDate: string;
  original: string;
  corrected: string;
  explanation: string;
}

export interface RecurringMistakeDetails {
  mistake: RecurringMistake;
  occurrences: MistakeOccurrence[];
}

export interface LearningMemorySyncResult {
  synchronized: number;
  failed: number;
  errors: string[];
}

export interface StudentLearningSummary {
  schemaVersion: number;
  generatedAt: string;
  analyzedLessonCount: number;
  completedLessonCount: number;
  recentStrengths: { title: string }[];
  currentFocusAreas: { area: string; title: string }[];
  confirmedRecurringMistakes: {
    id: string;
    title: string;
    category: string;
    lessonCount: number;
    occurrenceCount: number;
    exampleOriginal: string;
    exampleCorrected: string;
  }[];
  recentVocabulary: {
    id: string;
    text: string;
    meaning: string;
    status: VocabularyStatus;
  }[];
  nextLessonRecommendations: string[];
  latestPerformanceSnapshot: {
    lessonId: string;
    overall: number;
    fluency: number;
    grammar: number;
    vocabulary: number;
    comprehension: number;
    interaction: number;
  } | null;
}

export type VoiceEngineEvent =
  | { type: "engine_started" }
  | { type: "calibrating" }
  | { type: "calibrated"; voiceThreshold: number }
  | { type: "listening" }
  | { type: "student_speaking" }
  | { type: "speech_finished" }
  | { type: "transcribing" }
  | { type: "transcript"; text: string; message?: TranscriptMessage }
  | { type: "teacher_thinking" }
  | { type: "teacher_stream_started"; generationId: string }
  | {
      type: "teacher_response_delta";
      generationId: string;
      delta: string;
      text: string;
    }
  | { type: "teacher_chunk_ready"; generationId: string; chunkIndex: number }
  | {
      type: "teacher_playback_started";
      generationId: string;
      chunkIndex: number;
    }
  | {
      type: "teacher_response";
      text: string;
      generationId?: string;
      partial?: boolean;
      message?: TranscriptMessage;
      correctionCandidate?: CorrectionCandidate;
    }
  | { type: "teacher_speaking"; generationId?: string }
  | { type: "teacher_cancel_requested"; requested: boolean }
  | { type: "teacher_cancelled"; generationId: string; deliveredText: string }
  | { type: "streaming_fallback"; generationId: string; reason: string }
  | ({ type: "voice_turn_metrics" } & VoiceTurnPerformance)
  | { type: "teacher_finished"; generationId?: string }
  | { type: "error"; message: string; recoverable?: boolean }
  | { type: "engine_stopped" };

export interface VoiceTurnPerformance {
  turnId: string;
  runtimeVersion: number;
  streamingEnabled: boolean;
  sttMs: number | null;
  llmTtftMs: number | null;
  llmFirstSentenceMs: number | null;
  llmTotalMs: number | null;
  firstTtsMs: number | null;
  speechEndToFirstAudioMs: number | null;
  lastVoiceToFirstAudioMs: number | null;
  captureEndToFirstAudioMs: number | null;
  ttsTotalMs: number | null;
  teacherPlaybackMs: number | null;
  teacherTurnTotalMs: number | null;
  ttsChunkCount: number;
  cancelled: boolean;
  fallbackUsed: boolean;
  createdAt: string;
}

export interface PipelineMetrics {
  sttMs: number;
  llmMs: number;
  ttsMs: number;
  totalMs: number;
}

export interface PipelineResult {
  studentText: string;
  teacherText: string;
  speechAudioBase64: string | null;
  audioMimeType: string | null;
  metrics: PipelineMetrics;
}

export interface OllamaModel {
  name: string;
  size: number;
  modifiedAt: string;
}

export type CefrBand = "A1" | "A2" | "B1" | "B2" | "C1" | "C2";
export type PlacementConfidence = "low" | "medium" | "high";
export type PlacementAttemptStatus =
  "in_progress" | "completed" | "abandoned" | "failed";
export type PlacementSpeakingStatus =
  "pending" | "completed" | "skipped" | "unavailable";
export type PlacementSkill = "grammar" | "vocabulary" | "reading";

export interface PlacementAttempt {
  id: string;
  status: PlacementAttemptStatus;
  testVersion: number;
  questionBankVersion: number;
  scoringVersion: number;
  speakingPromptVersion: number;
  speakingEvaluatorVersion: number | null;
  speakingSchemaVersion: number | null;
  startedAt: string;
  completedAt: string | null;
  grammarLevel: CefrBand | null;
  vocabularyLevel: CefrBand | null;
  readingLevel: CefrBand | null;
  spokenProductionLevel: CefrBand | null;
  overallEstimatedLevel: CefrBand | null;
  confidence: PlacementConfidence | null;
  speakingStatus: PlacementSpeakingStatus;
  errorMessage: string | null;
}

export interface PlacementOption {
  id: string;
  text: string;
}
export interface PlacementQuestion {
  questionId: string;
  skill: PlacementSkill;
  prompt: string;
  options: PlacementOption[];
  passage: string | null;
}
export interface PlacementSpeakingPrompt {
  promptId: string;
  promptVersion: number;
  sequenceIndex: number;
  prompt: string;
}
export interface PlacementDomainProgress {
  skill: PlacementSkill;
  status: "pending" | "in_progress" | "complete";
  estimatedLevel: CefrBand | null;
  answeredQuestions: number;
}
export interface PlacementProgress {
  domains: PlacementDomainProgress[];
  phase: "objective" | "speaking" | "ready_to_finalize";
  speakingResponses: number;
  speakingWordCount: number;
}
export interface PlacementSession {
  attempt: PlacementAttempt;
  progress: PlacementProgress;
  question: PlacementQuestion | null;
  speakingPrompt: PlacementSpeakingPrompt | null;
}
export interface PlacementDomainResult {
  skill: PlacementSkill | "spoken_production";
  level: CefrBand | null;
  assessed: boolean;
}
export interface PlacementSpeakingEvidence {
  criterion: string;
  observation: string;
  example: string;
}
export interface PlacementResult {
  attempt: PlacementAttempt;
  estimatedCefrLevel: CefrBand;
  confidence: PlacementConfidence;
  domains: PlacementDomainResult[];
  speakingEvidence: PlacementSpeakingEvidence[];
  speakingSummary: string | null;
  listeningAssessed: false;
  pronunciationAssessed: false;
  writingAssessed: false;
  disclaimer: string;
}
export interface PlacementOverview {
  activeAttempt: PlacementAttempt | null;
  currentResult: PlacementResult | null;
  attemptCount: number;
}

export type LearningGoal =
  | "general_fluency"
  | "everyday_conversation"
  | "travel_english"
  | "professional_english"
  | "job_interview"
  | "academic_english"
  | "grammar_accuracy"
  | "vocabulary_growth"
  | "speaking_confidence"
  | "exam_preparation";
export interface CurrentPlacementProfile {
  attemptId: string;
  estimatedLevel: CefrBand;
  confidence: PlacementConfidence;
  assessedAt: string;
}
export interface StudentLearningProfile {
  schemaVersion: number;
  currentPlacement: CurrentPlacementProfile | null;
  targetLevel: CefrBand | null;
  learningGoals: LearningGoal[];
  defaultLessonDifficulty: LessonDifficulty;
  useProfileInLessons: boolean;
}
export interface UpdateStudentProfileRequest {
  targetLevel: CefrBand | null;
  learningGoals: LearningGoal[];
  defaultLessonDifficulty: LessonDifficulty;
  useProfileInLessons: boolean;
}
export interface StudentProfileContextStatus {
  enabled: boolean;
  placementAvailable: boolean;
  contextAvailable: boolean;
  contextVersion: number;
}
export interface LessonStudentProfileSnapshot {
  lessonId: string;
  profileSchemaVersion: number;
  profileContextVersion: number;
  contextEnabled: boolean;
  placementAttemptId: string | null;
  estimatedCefrLevel: CefrBand | null;
  placementConfidence: PlacementConfidence | null;
  targetCefrLevel: CefrBand | null;
  learningGoals: LearningGoal[];
  lessonDifficulty: LessonDifficulty;
  createdAt: string;
}

export interface WeeklyGoalProgress {
  goalMinutes: number;
  practicedMinutes: number;
  progressPercent: number;
  reached: boolean;
}
export interface GamificationOverview {
  schemaVersion: number;
  totalXp: number;
  practiceLevel: number;
  currentLevelThreshold: number;
  nextLevelThreshold: number;
  xpIntoCurrentLevel: number;
  xpNeededForNextLevel: number;
  qualifyingLessonCount: number;
  totalPracticeMinutes: number;
  currentStreakDays: number;
  longestStreakDays: number;
  weeklyGoal: WeeklyGoalProgress;
  unlockedAchievementCount: number;
  totalAchievementCount: number;
}
export interface Achievement {
  id: string;
  version: number;
  title: string;
  description: string;
  category: string;
  unlocked: boolean;
  unlockedAt: string | null;
  progressCurrent: number;
  progressTarget: number;
}
export interface GamificationProfile {
  schemaVersion: number;
  weeklyGoalMinutes: number;
}
export interface GamificationSyncResult {
  inspectedLessons: number;
  qualifyingLessons: number;
  ignoredLessons: number;
  eventsCreated: number;
  achievementsUnlocked: Achievement[];
}

export type ReviewMode = "mixed" | "vocabulary" | "mistakes";
export type ReviewOutcome =
  | "keep_practicing"
  | "mark_learning"
  | "mark_known"
  | "review_again"
  | "reviewed";
export interface ReviewSessionSummary {
  id: string;
  status: "in_progress" | "completed" | "abandoned" | "failed";
  mode: ReviewMode;
  requestedItemCount: number;
  actualItemCount: number;
  reviewedItemCount: number;
  startedAt: string;
  completedAt: string | null;
  abandonedAt: string | null;
}
export interface VocabularyReviewContent {
  schemaVersion: number;
  displayText: string;
  meaning: string;
  example: string | null;
  statusAtStart: "new" | "learning";
  lessonCount: number;
  occurrenceCount: number;
}
export interface MistakeReviewContent {
  schemaVersion: number;
  title: string;
  category: string;
  original: string;
  corrected: string;
  explanation: string;
  lessonCount: number;
  occurrenceCount: number;
}
export type ReviewItem =
  | {
      type: "vocabulary";
      id: string;
      sequenceIndex: number;
      reviewed: boolean;
      reviewOutcome: ReviewOutcome | null;
      reviewedAt: string | null;
      content: VocabularyReviewContent;
    }
  | {
      type: "recurring_mistake";
      id: string;
      sequenceIndex: number;
      reviewed: boolean;
      reviewOutcome: ReviewOutcome | null;
      reviewedAt: string | null;
      content: MistakeReviewContent;
    };
export interface ReviewCompletionSummary {
  itemsReviewed: number;
  vocabularyReviewed: number;
  mistakesReviewed: number;
  vocabularyMarkedLearning: number;
  vocabularyMarkedKnown: number;
}
export interface ReviewSession {
  id: string;
  status: ReviewSessionSummary["status"];
  mode: ReviewMode;
  requestedItemCount: number;
  actualItemCount: number;
  reviewedItemCount: number;
  currentIndex: number;
  startedAt: string;
  completedAt: string | null;
  currentItem: ReviewItem | null;
  completionSummary: ReviewCompletionSummary | null;
}
export interface ReviewHistory {
  completedSessionCount: number;
  reviewedItemCount: number;
  reviewedThisWeek: number;
  vocabularyReviewed: number;
  mistakesReviewed: number;
  lastReviewAt: string | null;
}
export interface ReviewOverview {
  schemaVersion: number;
  activeSession: ReviewSessionSummary | null;
  vocabulary: {
    newCount: number;
    learningCount: number;
    totalEligibleCount: number;
  };
  recurringMistakes: { confirmedCount: number };
  reviewHistory: ReviewHistory;
  suggestedFocus: string | null;
  recentSessions: ReviewSessionSummary[];
}
export interface StartReviewSessionRequest {
  mode: ReviewMode;
  itemCount: 5 | 10 | 15;
  startOver?: boolean;
}
export interface ReviewSubmitResult {
  session: ReviewSession;
  vocabularyStatusChanged: boolean;
}

export type PronunciationStatus =
  | "completed"
  | "content_mismatch"
  | "insufficient_audio"
  | "alignment_failed"
  | "engine_unavailable"
  | "cancelled"
  | "failed";
export type PronunciationConfidence = "low" | "medium" | "high";
export interface PronunciationPhoneResult {
  phone: string;
  score: number;
  startMs: number;
  endMs: number;
  frameCount: number;
  closestAlternative: string | null;
  hint: string | null;
}
export interface PronunciationWordResult {
  index: number;
  word: string;
  score: number;
  startMs: number;
  endMs: number;
  expectedPhones: string[];
  phoneResults: PronunciationPhoneResult[];
}
export interface PronunciationAttempt {
  id: string;
  status: PronunciationStatus;
  sourceType: "custom" | "vocabulary" | "diagnostic" | "interactive_lesson";
  sourceId: string | null;
  targetText: string;
  locale: "en-US";
  overallScore: number | null;
  confidence: PronunciationConfidence | null;
  contentMatchScore: number | null;
  alignmentCoverage: number | null;
  audioDurationMs: number | null;
  createdAt: string;
  completedAt: string | null;
  words: PronunciationWordResult[];
}
export interface PronunciationEngineStatus {
  installed: boolean;
  available: boolean;
  ready: boolean;
  engineVersion: number;
  scoreVersion: number;
  resultSchemaVersion: number;
  modelId: string;
  modelRevision: string;
  phonemizerReady: boolean;
  loadMs: number | null;
  lastError: string | null;
}
export interface ReviewQueuePreview {
  requestedItemCount: number;
  actualItemCount: number;
  mistakes: number;
  learningVocabulary: number;
  newVocabulary: number;
}

export type DiagnosticStatus = "healthy" | "warning" | "unavailable";
export interface DiagnosticComponent {
  status: DiagnosticStatus;
  version: string | null;
  message: string;
  technicalCode: string | null;
  advancedDetails: Record<string, unknown>;
}
export interface SystemDiagnostics {
  reportVersion: number;
  generatedAt: string;
  appVersion: string;
  platform: string;
  database: DiagnosticComponent;
  ollama: DiagnosticComponent;
  whisper: DiagnosticComponent;
  piper: DiagnosticComponent;
  voiceBridge: DiagnosticComponent;
  voiceStreaming: DiagnosticComponent;
  pronunciation: DiagnosticComponent;
  settings: DiagnosticComponent;
  overallStatus: string;
  conversationReady: boolean;
  pronunciationReady: boolean;
  databaseReady: boolean;
}
export interface BackupSummary {
  backupId: string;
  createdAt: string;
  path: string;
  databaseBytes: number;
  databaseSha256: string;
  schemaVersion: number;
  valid: boolean;
}
export interface BackupValidation {
  valid: boolean;
  backupId: string;
  schemaVersion: number | null;
  integrity: string;
  foreignKeyViolations: number;
  message: string;
}
export interface RestoreResultRecord {
  status: "completed" | "failed";
  backupId: string;
  message: string;
  occurredAt: string;
}
export interface BackupStatus {
  operation:
    | "idle"
    | "creating"
    | "validating"
    | "creating_safety_backup"
    | "restoring"
    | "reopening"
    | "completed"
    | "failed";
  error: string | null;
  backupDirectory: string;
  lastBackup: BackupSummary | null;
  restoreAllowed: boolean;
  restoreBlockReason: string | null;
  pendingRestart: boolean;
  lastRestore: RestoreResultRecord | null;
}
export interface RestoreScheduled {
  backupId: string;
  safetyBackupId: string;
  restartRequired: boolean;
  message: string;
}
export interface SystemEvent {
  id: string;
  severity: "warning" | "error" | "recovery";
  component: string;
  eventCode: string;
  details: Record<string, unknown> | null;
  occurredAt: string;
}
export interface WelcomeState {
  shouldShow: boolean;
  hasSeen: boolean;
  existingUser: boolean;
}

export type InteractiveStageType =
  | "theory"
  | "visual_vocabulary"
  | "listening"
  | "repeat"
  | "speaking_check"
  | "exercise"
  | "guided_conversation"
  | "analysis";
export type InteractiveSessionStatus =
  "in_progress" | "completed" | "abandoned" | "failed";
export type InteractiveStageStatus =
  "pending" | "active" | "completed" | "skipped";
export interface GuidedStageCapability {
  stageType: InteractiveStageType;
  stageSchemaVersion: number;
  runtimeAvailable: boolean;
}
export interface GuidedLessonSummary {
  lessonId: string;
  contentVersion: number;
  title: string;
  description: string;
  cefrBand: CefrBand;
  estimatedMinutes: number;
  objectives: string[];
  tags: string[];
  stageCount: number;
  startable: boolean;
  unavailableReasons: string[];
}
export interface GuidedStageOverview {
  stageId: string;
  stageType: InteractiveStageType;
  title: string;
  required: boolean;
  available: boolean;
}
export interface GuidedLessonDetail extends GuidedLessonSummary {
  stageOverview: GuidedStageOverview[];
}
export interface GuidedSessionStage {
  stageId: string;
  sequenceIndex: number;
  stageType: InteractiveStageType;
  title: string;
  required: boolean;
  status: InteractiveStageStatus;
  attemptCount: number;
}
export interface TheoryBlock {
  type: "paragraph" | "bullet_list" | "example" | "callout";
  text?: string | null;
  items?: string[] | null;
  english?: string | null;
  explanation?: string | null;
  title?: string | null;
}
export interface VisualVocabularyItem {
  itemId: string;
  term: string;
  meaning: string;
  example: string;
  imageAssetId: string | null;
}
export interface GuidedPronunciationAttempt {
  id: string;
  attemptIndex: number;
  status: PronunciationStatus | "analyzing";
  selected: boolean;
  result: PronunciationAttempt | null;
  createdAt: string;
  completedAt: string | null;
}
export interface GuidedListeningSegment {
  segmentId: string;
  text: string;
  hasBundledAudio: boolean;
  completedPlaybackCount: number;
}
export interface GuidedRepeatTarget {
  targetId: string;
  text: string;
  hint: string | null;
  hasBundledAudio: boolean;
  completedReferencePlaybackCount: number;
  selectedAttemptId: string | null;
  attempts: GuidedPronunciationAttempt[];
}
export interface GuidedSpeakingTarget {
  targetId: string;
  instruction: string;
  targetText: string;
  hint: string | null;
  selectedAttemptId: string | null;
  attempts: GuidedPronunciationAttempt[];
}
export type ExerciseType =
  | "single_choice"
  | "multiple_select"
  | "fill_blank"
  | "word_order"
  | "matching"
  | "short_answer_exact";
export interface ExerciseOption {
  optionId: string;
  text: string;
}
export interface ExerciseToken {
  tokenId: string;
  text: string;
}
export interface ExerciseMatchingItem {
  itemId: string;
  text: string;
}
export interface ExerciseMatchingPair {
  leftId: string;
  rightId: string;
}
export type ExerciseExpectedAnswer =
  | { kind: "single_choice"; option: ExerciseOption }
  | { kind: "multiple_select"; options: ExerciseOption[] }
  | { kind: "fill_blank"; answer: string }
  | { kind: "word_order"; tokens: ExerciseToken[] }
  | {
      kind: "matching";
      pairs: { left: ExerciseMatchingItem; right: ExerciseMatchingItem }[];
    }
  | { kind: "short_answer_exact"; answer: string };
export interface GuidedExerciseAttempt {
  attemptId: string;
  attemptIndex: number;
  correct: boolean;
  selected: boolean;
  feedback: string;
  explanation: string | null;
  expectedAnswer: ExerciseExpectedAnswer;
  normalizationVersion: number;
  submittedAt: string;
  selectedAt: string | null;
}
interface GuidedExerciseItemBase {
  exerciseId: string;
  prompt: string;
  instructions: string | null;
  hint: string | null;
  attempts: GuidedExerciseAttempt[];
  selectedAttemptId: string | null;
}
export type GuidedExerciseItem =
  | (GuidedExerciseItemBase & {
      exerciseType: "single_choice" | "multiple_select";
      payload: { options: ExerciseOption[] };
    })
  | (GuidedExerciseItemBase & {
      exerciseType: "fill_blank";
      payload: { prefix: string; suffix: string };
    })
  | (GuidedExerciseItemBase & {
      exerciseType: "word_order";
      payload: { tokens: ExerciseToken[] };
    })
  | (GuidedExerciseItemBase & {
      exerciseType: "matching";
      payload: {
        leftItems: ExerciseMatchingItem[];
        rightItems: ExerciseMatchingItem[];
      };
    })
  | (GuidedExerciseItemBase & {
      exerciseType: "short_answer_exact";
      payload: { normalizationProfile: "english_basic_v1" };
    });
export interface GuidedExerciseSummary {
  exerciseCount: number;
  selectedCorrectCount: number;
  selectedIncorrectCount: number;
  totalAttemptCount: number;
  accuracyPercent: number;
}
export interface GuidedExerciseStage {
  engineVersion: number;
  stageSchemaVersion: number;
  attemptResultVersion: number;
  normalizationVersion: number;
  currentExerciseIndex: number;
  items: GuidedExerciseItem[];
  summary: GuidedExerciseSummary | null;
}
export type ExerciseResponse =
  | { exerciseType: "single_choice"; value: { optionId: string } }
  | { exerciseType: "multiple_select"; value: { optionIds: string[] } }
  | { exerciseType: "fill_blank"; value: { text: string } }
  | { exerciseType: "word_order"; value: { tokenIds: string[] } }
  | { exerciseType: "matching"; value: { pairs: ExerciseMatchingPair[] } }
  | { exerciseType: "short_answer_exact"; value: { text: string } };
export interface GuidedConversationTurn {
  id: string;
  sequenceIndex: number;
  role: "student" | "assistant";
  text: string;
  partial: boolean;
  createdAt: string;
}
export type GuidedActiveContent =
  | { kind: "theory"; blocks: TheoryBlock[] }
  | { kind: "visual_vocabulary"; items: VisualVocabularyItem[] }
  | {
      kind: "listening";
      segments: GuidedListeningSegment[];
      revealTextAfterFirstPlay: boolean;
    }
  | { kind: "repeat"; targets: GuidedRepeatTarget[] }
  | { kind: "speaking_check"; targets: GuidedSpeakingTarget[] }
  | { kind: "exercise"; stage: GuidedExerciseStage }
  | {
      kind: "guided_conversation";
      scenario: string;
      studentRole: string;
      teacherRole: string;
      goal: string;
      targetVocabulary: string[];
      targetExpressions: string[];
      minimumStudentTurns: number;
      recommendedStudentTurns: number;
      maximumStudentTurns: number;
      started: boolean;
      studentTurnCount: number;
      assistantTurnCount: number;
      turns: GuidedConversationTurn[];
    };
export interface GuidedActiveStage {
  stageId: string;
  sequenceIndex: number;
  stageType: InteractiveStageType;
  title: string;
  instructions: string;
  required: boolean;
  content: GuidedActiveContent;
}
export interface GuidedLessonSession {
  id: string;
  lessonId: string;
  contentVersion: number;
  title: string;
  cefrBand: CefrBand;
  status: InteractiveSessionStatus;
  currentStageIndex: number;
  stageCount: number;
  progressPercent: number;
  stages: GuidedSessionStage[];
  activeStage: GuidedActiveStage | null;
  startedAt: string;
  completedAt: string | null;
  abandonedAt: string | null;
}
export interface GuidedLessonOverview {
  publishedLessonCount: number;
  activeSession: GuidedLessonSession | null;
  capabilities: GuidedStageCapability[];
}
export interface GuidedAudio {
  playbackId: string;
  audioBase64: string;
  mimeType: "audio/wav";
  source: "bundled" | "piper" | "piper_cache";
  durationMs: number;
  runtimeVersion: number;
}
