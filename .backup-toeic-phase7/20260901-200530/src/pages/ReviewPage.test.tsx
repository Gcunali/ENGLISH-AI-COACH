// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { ReviewOverview } from '../types'
const mocks=vi.hoisted(()=>({overview:vi.fn(),start:vi.fn()}))
vi.mock('../hooks/useReviewData',()=>({useReviewOverview:mocks.overview}))
vi.mock('../services/native',()=>({startReviewSession:mocks.start}))
import { ReviewPage } from './ReviewPage'
const reload=vi.fn()
const base:ReviewOverview={schemaVersion:1,activeSession:null,vocabulary:{newCount:4,learningCount:6,totalEligibleCount:10},recurringMistakes:{confirmedCount:2},reviewHistory:{completedSessionCount:1,reviewedItemCount:5,reviewedThisWeek:5,vocabularyReviewed:4,mistakesReviewed:1,lastReviewAt:'2026-08-20T12:00:00Z'},suggestedFocus:'Practice prepositions.',recentSessions:[{id:'old',status:'completed',mode:'mixed',requestedItemCount:5,actualItemCount:5,reviewedItemCount:5,startedAt:'2026-08-20T12:00:00Z',completedAt:'2026-08-20T12:05:00Z',abandonedAt:null}]}
function view(path='/review'){return render(<MemoryRouter initialEntries={[path]}><Routes><Route path="/review" element={<ReviewPage/>}/><Route path="/review/session/:id" element={<div>Session destination</div>}/></Routes></MemoryRouter>)}
describe('ReviewPage',()=>{beforeEach(()=>{vi.clearAllMocks();mocks.overview.mockReturnValue({data:base,loading:false,error:null,reload});mocks.start.mockResolvedValue({id:'new-session'})});afterEach(cleanup)
 it('shows real counts, selectors, focus, history and starts with defaults',async()=>{view();expect(screen.getByText('Practice prepositions.')).toBeInTheDocument();expect(screen.getByText('12 eligible items')).toBeInTheDocument();expect(screen.getByLabelText('Mixed Review')).toBeChecked();expect(screen.getByLabelText('10 items')).toBeChecked();expect(screen.getByText('Recent Review Sessions')).toBeInTheDocument();fireEvent.click(screen.getByRole('button',{name:'Start Review'}));await waitFor(()=>expect(mocks.start).toHaveBeenCalledWith({mode:'mixed',itemCount:10,startOver:false}));expect(await screen.findByText('Session destination')).toBeInTheDocument()})
 it('uses query mode and blocks an empty source',()=>{mocks.overview.mockReturnValue({data:{...base,recurringMistakes:{confirmedCount:0}},loading:false,error:null,reload});view('/review?mode=mistakes');expect(screen.getByLabelText('Recurring Mistakes')).toBeChecked();expect(screen.getByText('Nothing needs review yet for this mode.')).toBeInTheDocument();expect(screen.queryByRole('button',{name:'Start Review'})).not.toBeInTheDocument()})
 it('offers resume and confirmed start over for an active session',async()=>{mocks.overview.mockReturnValue({data:{...base,activeSession:{id:'active',status:'in_progress',mode:'mixed',requestedItemCount:10,actualItemCount:8,reviewedItemCount:3,startedAt:'2026-08-21',completedAt:null,abandonedAt:null}},loading:false,error:null,reload});view();expect(screen.getByRole('button',{name:'Resume Review'})).toBeInTheDocument();fireEvent.click(screen.getByRole('button',{name:'Start Over'}));const dialog=screen.getByRole('dialog',{name:'Start this Review over?'});expect(dialog).toBeInTheDocument();fireEvent.click(within(dialog).getByRole('button',{name:'Start Over'}));await waitFor(()=>expect(mocks.start).toHaveBeenCalledWith({mode:'mixed',itemCount:10,startOver:true}))})
})
