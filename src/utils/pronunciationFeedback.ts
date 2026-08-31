import type { PronunciationAttempt, PronunciationWordResult } from '../types'

export const WORD_PRONUNCIATION_FEEDBACK_VERSION = 1 as const
export type WordFeedbackLabel = 'Strong' | 'Good' | 'Needs attention'
export interface WordFeedback { word: PronunciationWordResult; label: WordFeedbackLabel; focus: boolean }

export function wordFeedback(attempt: PronunciationAttempt): { available: boolean; reason: string | null; items: WordFeedback[] } {
  if (attempt.status !== 'completed' || attempt.confidence === 'low' || (attempt.alignmentCoverage ?? 0) < 0.8 || attempt.words.length === 0) {
    return { available: false, reason: 'Word-level alignment was not reliable enough for specific feedback. Try again in a quieter setting and keep the overall result as a gentle signal.', items: [] }
  }
  const focusIndexes = new Set([...attempt.words].sort((a, b) => a.score - b.score).filter(word => word.score < 70).slice(0, 3).map(word => word.index))
  return { available: true, reason: null, items: attempt.words.map(word => ({ word, label: word.score >= 85 ? 'Strong' : word.score >= 70 ? 'Good' : 'Needs attention', focus: focusIndexes.has(word.index) })) }
}
