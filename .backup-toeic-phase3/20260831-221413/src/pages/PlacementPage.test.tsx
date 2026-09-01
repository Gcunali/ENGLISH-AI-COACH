// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { PlacementOverview, PlacementSession } from '../types'

const mocks = vi.hoisted(() => ({
  useOverview: vi.fn(), start: vi.fn(), resume: vi.fn(), abandon: vi.fn(), submit: vi.fn(), confirm: vi.fn(), skip: vi.fn(), finalize: vi.fn(),
  recorder: { state: 'idle', transcript: '', error: null, start: vi.fn(), stop: vi.fn(), retry: vi.fn() },
}))
vi.mock('../hooks/usePlacementOverview', () => ({ usePlacementOverview: mocks.useOverview }))
vi.mock('../hooks/usePlacementRecorder', () => ({ usePlacementRecorder: () => mocks.recorder }))
vi.mock('../services/native', () => ({
  startPlacementTest: mocks.start, resumePlacementTest: mocks.resume, abandonPlacementTest: mocks.abandon,
  submitPlacementAnswer: mocks.submit, confirmPlacementSpeakingResponse: mocks.confirm,
  skipPlacementSpeaking: mocks.skip, finalizePlacementTest: mocks.finalize,
}))
import { PlacementPage } from './PlacementPage'

const attempt = { id:'attempt-1', status:'in_progress', testVersion:1, questionBankVersion:1, scoringVersion:1, speakingPromptVersion:1, speakingEvaluatorVersion:null, speakingSchemaVersion:null, startedAt:'2026-08-21T10:00:00Z', completedAt:null, grammarLevel:null, vocabularyLevel:null, readingLevel:null, spokenProductionLevel:null, overallEstimatedLevel:null, confidence:null, speakingStatus:'pending', errorMessage:null } as const
const questionSession: PlacementSession = { attempt, progress:{phase:'objective',speakingResponses:0,speakingWordCount:0,domains:[{skill:'grammar',status:'in_progress',estimatedLevel:null,answeredQuestions:0},{skill:'vocabulary',status:'pending',estimatedLevel:null,answeredQuestions:0},{skill:'reading',status:'pending',estimatedLevel:null,answeredQuestions:0}]}, question:{questionId:'reading-b1-1',skill:'reading',prompt:'What is the main idea?',passage:'A short reading passage.',options:[{id:'a',text:'First option'},{id:'b',text:'Second option'}]}, speakingPrompt:null }
const overview: PlacementOverview = { activeAttempt:null,currentResult:null,attemptCount:0 }

describe('PlacementPage', () => {
  beforeEach(() => { vi.clearAllMocks(); mocks.useOverview.mockReturnValue({data:overview,loading:false,error:null,reload:vi.fn()}); Object.assign(mocks.recorder,{state:'idle',transcript:'',error:null}) })
  afterEach(cleanup)
  it('renders transparent landing with no previous attempt', () => { render(<MemoryRouter><PlacementPage /></MemoryRouter>); expect(screen.getByRole('heading',{name:'Placement Test'})).toBeInTheDocument(); expect(screen.getByText('Listening, pronunciation, and formal writing are not assessed.',{exact:false})).toBeInTheDocument(); expect(screen.getByRole('button',{name:'Start Placement Test'})).toBeEnabled() })
  it('offers resume and confirmed start over for an active attempt', () => { mocks.useOverview.mockReturnValue({data:{...overview,activeAttempt:attempt},loading:false,error:null,reload:vi.fn()}); render(<MemoryRouter><PlacementPage /></MemoryRouter>); expect(screen.getByRole('button',{name:'Resume Placement'})).toBeInTheDocument(); fireEvent.click(screen.getByRole('button',{name:'Start Over'})); expect(screen.getByRole('dialog',{name:'Start the Placement Test over?'})).toBeInTheDocument() })
  it('renders one question and reading passage without answer feedback', async () => { mocks.start.mockResolvedValue(questionSession); mocks.submit.mockResolvedValue(questionSession); render(<MemoryRouter><PlacementPage /></MemoryRouter>); fireEvent.click(screen.getByRole('button',{name:'Start Placement Test'})); expect(await screen.findByText('A short reading passage.')).toBeInTheDocument(); fireEvent.click(screen.getByLabelText('First option')); fireEvent.click(screen.getByRole('button',{name:'Submit answer'})); await waitFor(()=>expect(mocks.submit).toHaveBeenCalledWith('attempt-1','reading-b1-1','a')); expect(screen.queryByText(/correct|wrong/i)).not.toBeInTheDocument() })
  it('shows speaking prompt, transcript preview, retry and skip confirmation', async () => { const speaking={...questionSession,progress:{...questionSession.progress,phase:'speaking' as const,speakingResponses:1,speakingWordCount:22},question:null,speakingPrompt:{promptId:'speaking-2',promptVersion:1,sequenceIndex:1,prompt:'Tell us about a changed plan.'}}; mocks.start.mockResolvedValue(speaking); Object.assign(mocks.recorder,{transcript:'My plan changed and I adapted to the situation.',state:'idle'}); render(<MemoryRouter><PlacementPage /></MemoryRouter>); fireEvent.click(screen.getByRole('button',{name:'Start Placement Test'})); expect(await screen.findByText('Tell us about a changed plan.')).toBeInTheDocument(); expect(screen.getByLabelText('Transcript preview')).toBeInTheDocument(); fireEvent.click(screen.getByRole('button',{name:'Retry'})); expect(mocks.recorder.retry).toHaveBeenCalled(); fireEvent.click(screen.getByRole('button',{name:'Skip speaking section'})); expect(screen.getByRole('dialog',{name:'Skip Spoken Production?'})).toBeInTheDocument() })
})
