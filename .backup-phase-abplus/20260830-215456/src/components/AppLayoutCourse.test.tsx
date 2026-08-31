// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest'
import { cleanup, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const getCourseCatalog=vi.fn()
vi.mock('../services/native',()=>({
  getCourseCatalog:(...args:unknown[])=>getCourseCatalog(...args),
  getGuidedLessonOverview:()=>Promise.resolve({publishedLessonCount:0,activeSession:null,capabilities:[]}),
  probeLocalVoiceEngine:()=>Promise.resolve({offlineReady:true}),
  subscribeGamificationChanges:()=>Promise.resolve(()=>undefined),
  subscribeReviewChanges:()=>Promise.resolve(()=>undefined),
}))
vi.mock('../utils/gamificationData',()=>({notifyGamificationDataChanged:vi.fn()}))
vi.mock('../utils/reviewData',()=>({notifyReviewDataChanged:vi.fn()}))
import { AppLayout } from './AppLayout'

beforeEach(()=>getCourseCatalog.mockReset())
afterEach(cleanup)
function view(){return render(<MemoryRouter initialEntries={['/']}><Routes><Route element={<AppLayout/>}><Route index element={<div>Home</div>}/></Route></Routes></MemoryRouter>)}
describe('Course navigation visibility',()=>{
  it('hides Course when no published Curriculum exists',async()=>{getCourseCatalog.mockResolvedValue({publishedCurriculumCount:0,curricula:[]});view();await waitFor(()=>expect(getCourseCatalog).toHaveBeenCalledOnce());expect(screen.queryByRole('link',{name:'Course'})).not.toBeInTheDocument()})
  it('shows Course while preserving the distinct Guided Lessons rule when a Curriculum exists',async()=>{getCourseCatalog.mockResolvedValue({publishedCurriculumCount:1,curricula:[]});view();expect((await screen.findAllByRole('link',{name:'Course'})).length).toBeGreaterThan(0);expect(screen.queryByRole('link',{name:'Guided Lessons'})).not.toBeInTheDocument()})
})
