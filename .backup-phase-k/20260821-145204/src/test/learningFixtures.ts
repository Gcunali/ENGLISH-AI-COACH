import type { DashboardSummary, LearningMemorySummary, Lesson, LessonAnalysis, LessonDetails, LessonHistoryItem, ProgressOverview, RecurringMistake, RecurringMistakeDetails, StudentLearningSummary, TranscriptMessage, VocabularyItemDetails, VocabularyPage, VocabularySummary } from '../types'

export const lesson: Lesson = {
  id: 'lesson-1', startedAt: '2026-08-17T19:44:27.388Z', endedAt: '2026-08-17T19:46:08.388Z',
  status: 'completed', topic: null, mode: 'free_conversation', durationSeconds: 101,
  studentTurnCount: 3, teacherTurnCount: 3, correctionCount: 1, whisperModel: 'ggml-small.en-q5_1.bin',
  whisperThreads: 12, ollamaModel: 'qwen3.5:4b', piperVoice: 'en_US-lessac-medium',
  voiceEngineVersion: 'voice_v2_bridge_v1', errorMessage: null,
  createdAt: '2026-08-17T19:44:27.388Z', updatedAt: '2026-08-17T19:46:08.388Z',
}

export const analysis: LessonAnalysis = {
  id: 'analysis-1', lessonId: lesson.id, status: 'completed', schemaVersion: 1, promptVersion: 1,
  analyzerModel: 'qwen3.5:4b', startedAt: lesson.startedAt, completedAt: lesson.endedAt,
  overallScore: 81, scores: { fluency: 85, grammar: 70, vocabulary: 80, comprehension: 90, interaction: 80, pronunciation: null },
  strengths: [{ title: 'Boa abertura e engajamento', evidence: 'Hello teacher' }],
  priorityImprovements: [{ area: 'word_choice', title: 'PreposiÃ§Ã£o natural', explanation: "Use 'terrible at'.", exampleFromLesson: 'I am terrible cooking.', betterAlternative: "I'm terrible at cooking." }],
  corrections: [{ original: 'I am terrible cooking.', corrected: "I'm terrible at cooking.", explanation: "Use 'at'.", category: 'preposition' }],
  naturalAlternatives: [], vocabulary: [{ wordOrPhrase: 'terrible at', meaning: 'muito ruim em', example: "I'm terrible at math." }], recurringPatterns: [],
  nextLessonRecommendations: ['Praticar preposiÃ§Ãµes.'], summary: 'Resumo real da aula.', pronunciationAvailable: false,
  errorMessage: null, createdAt: lesson.startedAt, updatedAt: lesson.endedAt ?? lesson.startedAt,
}

export const historyItem: LessonHistoryItem = {
  id: lesson.id, startedAt: lesson.startedAt, endedAt: lesson.endedAt, durationSeconds: 101,
  status: 'completed', topic: null, mode: 'free_conversation', modeId: 'free_conversation', modeTitle: 'Free Conversation', customTitle: null, studentTurnCount: 3,
  teacherTurnCount: 3, correctionCount: 1, analysisStatus: 'completed', overallScore: 81,
}

export const dashboard: DashboardSummary = {
  totalLessons: 1, completedLessons: 1, totalPracticeSeconds: 101, totalStudentTurns: 3,
  totalCorrections: 1, analyzedLessons: 1, averageOverallScore: 81, latestLesson: historyItem,
  latestAnalyzedLesson: { lessonId: lesson.id, startedAt: lesson.startedAt, durationSeconds: 101, overallScore: 81, scores: analysis.scores! },
  latestRecommendation: 'Praticar preposiÃ§Ãµes.',
}

export const messages: TranscriptMessage[] = [
  { id: 'student-1', lessonId: lesson.id, sequenceIndex: 1, turnIndex: 1, role: 'student', text: 'I am terrible cooking.', source: 'whisper', engineEventType: 'transcript', createdAt: lesson.startedAt },
  { id: 'teacher-1', lessonId: lesson.id, sequenceIndex: 2, turnIndex: 1, role: 'teacher', text: "You can say: I'm terrible at cooking.", source: 'ollama', engineEventType: 'teacher_response', createdAt: lesson.startedAt },
]

export const details: LessonDetails = {
  lesson, configuration: { lessonId: lesson.id, modeId: 'free_conversation', modeVersion: 1, modeTitle: 'Free Conversation', lessonModeContextVersion: 1, difficulty: 'standard', topic: null, objective: null, scenario: null, focusAreas: [], customTitle: null, configurationSchemaVersion: 1, createdAt: lesson.createdAt, legacy: false }, messages, correctionCandidates: [{ id: 'correction-1', lessonId: lesson.id, studentMessageId: 'student-1', teacherMessageId: 'teacher-1', studentText: messages[0].text, teacherResponseText: messages[1].text, detectionMethod: 'teacher_cue_v1', createdAt: lesson.startedAt }], analysis,
}

export const progress: ProgressOverview = {
  analyzedLessonCount: 1, averages: { overall: 81, fluency: 85, grammar: 70, vocabulary: 80, comprehension: 90, interaction: 80 },
  strongestAreas: ['comprehension'], focusAreas: ['grammar'], latestRecommendation: 'Praticar preposiÃ§Ãµes.',
  points: [{ lessonId: lesson.id, date: lesson.startedAt, durationSeconds: 101, overall: 81, fluency: 85, grammar: 70, vocabulary: 80, comprehension: 90, interaction: 80 }],
}

export const vocabularySummary: VocabularySummary = { total: 1, new: 1, learning: 0, known: 0, contributingLessons: 1 }
export const memorySummary: LearningMemorySummary = { vocabularyTotal: 1, vocabularyNew: 1, vocabularyLearning: 0, vocabularyKnown: 0, lessonsContributingVocabulary: 1, recurringMistakesConfirmed: 0 }
export const vocabularyPage: VocabularyPage = {
  total: 1, limit: 25, offset: 0,
  items: [{ id: 'vocabulary-1', text: 'terrible at', meaning: 'muito ruim em', status: 'new', firstSeenAt: lesson.startedAt, lastSeenAt: lesson.startedAt, lessonCount: 1, occurrenceCount: 1, latestExample: "I'm terrible at math." }],
}
export const vocabularyDetails: VocabularyItemDetails = {
  item: vocabularyPage.items[0],
  occurrences: [{ lessonId: lesson.id, lessonDate: lesson.startedAt, example: "I'm terrible at math.", occurrenceCount: 1 }],
}
export const recurringMistake: RecurringMistake = { id: 'mistake-1', category: 'preposition', title: 'Preposition: "terrible at cooking"', explanation: "Use 'at' after terrible.", lessonCount: 2, occurrenceCount: 2, firstSeenAt: lesson.startedAt, lastSeenAt: '2026-08-18T19:44:27.388Z', status: 'active' }
export const recurringMistakeDetails: RecurringMistakeDetails = {
  mistake: recurringMistake,
  occurrences: [
    { lessonId: lesson.id, lessonDate: lesson.startedAt, original: 'I am terrible cooking.', corrected: "I'm terrible at cooking.", explanation: "Use 'at'." },
    { lessonId: 'lesson-2', lessonDate: '2026-08-18T19:44:27.388Z', original: 'I am terrible cooking.', corrected: "I'm terrible at cooking.", explanation: "Use 'at'." },
  ],
}

export const studentLearningSummary: StudentLearningSummary = {
  schemaVersion: 1,
  generatedAt: '2026-08-18T20:00:00.000Z',
  analyzedLessonCount: 2,
  completedLessonCount: 2,
  recentStrengths: [{ title: 'Keeps the conversation moving' }],
  currentFocusAreas: [{ area: 'preposition', title: 'Use natural prepositions' }],
  confirmedRecurringMistakes: [],
  recentVocabulary: [{ id: 'vocabulary-1', text: 'terrible at', meaning: 'very bad at something', status: 'learning' }],
  nextLessonRecommendations: ['Practice prepositions in everyday topics.'],
  latestPerformanceSnapshot: { lessonId: 'lesson-2', overall: 81, fluency: 85, grammar: 70, vocabulary: 80, comprehension: 90, interaction: 80 },
}

