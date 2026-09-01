// @vitest-environment jsdom

import '@testing-library/jest-dom/vitest'
import { cleanup, render, screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { LessonAnalysis } from '../types'
import { LessonAnalysisReport } from './LessonAnalysisReport'

afterEach(cleanup)

const completed: LessonAnalysis = {
  id: 'analysis-1', lessonId: 'lesson-1', status: 'completed', schemaVersion: 1,
  promptVersion: 1, analyzerModel: 'qwen3.5:4b', startedAt: 'start', completedAt: 'end',
  overallScore: 74,
  scores: { fluency: 70, grammar: 62, vocabulary: 68, comprehension: 84, interaction: 86, pronunciation: null },
  strengths: [{ title: 'Boa compreensão', evidence: 'Respondeu ao tópico corretamente.' }],
  priorityImprovements: [{ area: 'grammar', title: 'Passado simples', explanation: 'Use o passado em ações concluídas.', exampleFromLesson: 'Today I play tennis.', betterAlternative: 'Today I played tennis.' }],
  corrections: [{ original: 'Today I play tennis.', corrected: 'Today I played tennis.', explanation: 'Use played.', category: 'verb_tense' }],
  naturalAlternatives: [{ original: 'I am happy.', alternative: "I'm really pleased." }],
  vocabulary: [{ wordOrPhrase: 'pleased', meaning: 'satisfeito', example: "I'm pleased with it." }],
  recurringPatterns: [{ pattern: 'Past tense', count: 2, explanation: 'Ocorreu duas vezes.' }],
  nextLessonRecommendations: ['Praticar passado simples.'], summary: 'Resumo baseado na aula.',
  pronunciationAvailable: false, errorMessage: null, createdAt: 'start', updatedAt: 'end',
}

describe('LessonAnalysisReport', () => {
  it.each([
    ['pending', 'Analyzing your lesson locally'],
    ['running', 'Analyzing your lesson locally'],
    ['insufficient_data', 'Not enough data'],
  ] as const)('renders %s state', (status, text) => {
    render(<LessonAnalysisReport analysis={null} status={status} error={null} onRetry={vi.fn()} />)
    expect(screen.getByText(new RegExp(text))).toBeInTheDocument()
  })

  it('renders failed state and retry', () => {
    render(<LessonAnalysisReport analysis={null} status="failed" error="bad JSON" onRetry={vi.fn()} />)
    expect(screen.getByText('Retry analysis')).toBeInTheDocument()
    expect(screen.getByText('bad JSON')).toBeInTheDocument()
  })

  it('renders scores, pronunciation, feedback, corrections and vocabulary', () => {
    render(<LessonAnalysisReport analysis={completed} status="completed" error={null} onRetry={vi.fn()} />)
    for (const text of ['74', 'Fluency', 'Grammar', 'Vocabulary', 'Comprehension', 'Interaction', 'Not evaluated yet', 'Strengths', 'Priority improvements', 'Corrections', 'Today I played tennis.', 'Vocabulary from this lesson', 'Recurring patterns', 'Recommended focus for next lesson']) {
      expect(screen.getAllByText(text, { exact: false }).length).toBeGreaterThan(0)
    }
  })
})
