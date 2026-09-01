// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { PracticePage } from './PracticePage'

const native=vi.hoisted(()=>({getPracticeAvailability:vi.fn(),startPracticeSession:vi.fn()}))
vi.mock('../services/native',()=>native)
beforeEach(()=>{native.getPracticeAvailability.mockResolvedValue({schemaVersion:1,dailyCount:3,dictationCount:0,shadowingCount:0,mistakeRepairCount:0,speakingRecallCount:0,confirmedMistakeCount:0});native.startPracticeSession.mockResolvedValue({id:'session-1'})})
describe('PracticePage',()=>{
  it('shows honest eligibility and confirmed-mistake empty state',async()=>{render(<MemoryRouter><PracticePage/></MemoryRouter>);expect(await screen.findByText('3 eligible items')).toBeInTheDocument();expect(screen.getByText(/intentionally empty/)).toBeInTheDocument();expect(screen.getAllByRole('button',{name:/Start/})[0]).toBeEnabled()})
  it('starts a persisted daily session with the controlled size',async()=>{render(<MemoryRouter initialEntries={['/practice']}><Routes><Route path="/practice" element={<PracticePage/>}/><Route path="/practice/session/:id" element={<div>Session opened</div>}/></Routes></MemoryRouter>);fireEvent.click((await screen.findAllByRole('button',{name:/Start/}))[0]);await waitFor(()=>expect(native.startPracticeSession).toHaveBeenCalledWith('daily',7))})
})
