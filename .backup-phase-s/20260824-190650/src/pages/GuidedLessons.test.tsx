// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import * as native from '../services/native'
import type { GuidedLessonSession } from '../types'
import { GuidedLessonsPage } from './GuidedLessonsPage'
import { GuidedLessonSessionPage } from './GuidedLessonSessionPage'

vi.mock('../services/native',()=>({
  getGuidedLessonOverview:vi.fn(),listGuidedLessons:vi.fn(),listRecentGuidedLessonSessions:vi.fn(),
  getGuidedLessonSession:vi.fn(),completeGuidedLessonStage:vi.fn(),skipGuidedLessonStage:vi.fn(),abandonGuidedLesson:vi.fn(),
}))

const base:GuidedLessonSession={id:'session-1',lessonId:'everyday-greetings-a1',contentVersion:1,title:'Everyday Greetings',cefrBand:'A1',status:'in_progress',currentStageIndex:0,stageCount:2,progressPercent:0,startedAt:'2026-08-24',completedAt:null,abandonedAt:null,stages:[{stageId:'theory',sequenceIndex:0,stageType:'theory',title:'Theory',required:true,status:'active',attemptCount:0},{stageId:'words',sequenceIndex:1,stageType:'visual_vocabulary',title:'Words',required:true,status:'pending',attemptCount:0}],activeStage:{stageId:'theory',sequenceIndex:0,stageType:'theory',title:'Theory',instructions:'Read this.',required:true,content:{kind:'theory',blocks:[{type:'paragraph',text:'Greetings open a conversation.'},{type:'example',english:'Hello!',explanation:'A neutral greeting.'}]}}}

afterEach(cleanup)
beforeEach(()=>{vi.resetAllMocks();vi.mocked(native.getGuidedLessonOverview).mockResolvedValue({publishedLessonCount:0,activeSession:null,capabilities:[]});vi.mocked(native.listGuidedLessons).mockResolvedValue([]);vi.mocked(native.listRecentGuidedLessonSessions).mockResolvedValue([])})

describe('Guided Lessons UI foundation',()=>{
  it('shows the exact production empty state without exposing roadmap content',async()=>{render(<MemoryRouter><GuidedLessonsPage/></MemoryRouter>);expect(await screen.findByRole('heading',{name:'No guided lessons are installed yet.'})).toBeInTheDocument();expect(screen.queryByText(/coming soon/i)).not.toBeInTheDocument()})
  it('renders typed theory and advances with one backend action',async()=>{vi.mocked(native.getGuidedLessonSession).mockResolvedValue(base);const vocabulary={...base,currentStageIndex:1,progressPercent:50,stages:[{...base.stages[0],status:'completed' as const,attemptCount:1},{...base.stages[1],status:'active' as const}],activeStage:{stageId:'words',sequenceIndex:1,stageType:'visual_vocabulary' as const,title:'Words',instructions:'Review.',required:true,content:{kind:'visual_vocabulary' as const,items:[{itemId:'hello',term:'Hello',meaning:'A greeting.',example:'Hello, Ana.',imageAssetId:null}]}}};vi.mocked(native.completeGuidedLessonStage).mockResolvedValue(vocabulary);render(<MemoryRouter initialEntries={['/guided-lessons/session/session-1']}><Routes><Route path="/guided-lessons/session/:sessionId" element={<GuidedLessonSessionPage/>}/></Routes></MemoryRouter>);expect(await screen.findByText('Greetings open a conversation.')).toBeInTheDocument();fireEvent.click(screen.getByRole('button',{name:'Continue'}));expect(await screen.findByRole('heading',{name:'Hello'})).toBeInTheDocument();expect(native.completeGuidedLessonStage).toHaveBeenCalledWith('session-1','theory')})
  it('renders completion without scores or rewards',async()=>{vi.mocked(native.getGuidedLessonSession).mockResolvedValue({...base,status:'completed',progressPercent:100,activeStage:null,completedAt:'2026-08-24'});render(<MemoryRouter initialEntries={['/guided-lessons/session/session-1']}><Routes><Route path="/guided-lessons/session/:sessionId" element={<GuidedLessonSessionPage/>}/></Routes></MemoryRouter>);expect(await screen.findByText(/there is no score, XP or analysis/i)).toBeInTheDocument()})
  it('shows a friendly state for an unknown session',async()=>{vi.mocked(native.getGuidedLessonSession).mockResolvedValue(null);render(<MemoryRouter initialEntries={['/guided-lessons/session/missing']}><Routes><Route path="/guided-lessons/session/:sessionId" element={<GuidedLessonSessionPage/>}/></Routes></MemoryRouter>);expect(await screen.findByRole('heading',{name:'Guided lesson session not found'})).toBeInTheDocument()})
})
