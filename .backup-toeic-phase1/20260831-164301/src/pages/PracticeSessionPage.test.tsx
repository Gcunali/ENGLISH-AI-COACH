// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest'
import { render, screen } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { PracticeSessionPage } from './PracticeSessionPage'

const native=vi.hoisted(()=>({getPracticeSession:vi.fn(),recordPracticeTime:vi.fn(),completePracticeSession:vi.fn(),completePracticeItem:vi.fn(),preparePracticeAudio:vi.fn(),analyzePronunciation:vi.fn(),transcribeAudio:vi.fn(),cancelGuidedLessonPronunciation:vi.fn()}))
vi.mock('../services/native',()=>native)
beforeEach(()=>native.getPracticeSession.mockResolvedValue({id:'done',mode:'daily',status:'completed',schemaVersion:1,selectionVersion:1,items:[{kind:'vocabulary',itemId:'v1',term:'hello',meaning:'a greeting',example:'Hello there',status:'learning',sourceLabel:'Your vocabulary'}],completedItemIds:['v1'],activeSeconds:30,xpAwarded:20,startedAt:'2026-01-01',completedAt:'2026-01-01'}))
describe('PracticeSessionPage',()=>{
  it('renders a saved completion without throwing or mutating it again',async()=>{render(<MemoryRouter initialEntries={['/practice/session/done']}><Routes><Route path="/practice/session/:sessionId" element={<PracticeSessionPage/>}/></Routes></MemoryRouter>);expect(await screen.findByText(/practice was saved/i)).toBeInTheDocument();expect(screen.getByText(/20 XP awarded once/)).toBeInTheDocument();expect(native.completePracticeSession).not.toHaveBeenCalled()})
})
