// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest'
import { cleanup, render, screen } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { afterEach, describe, expect, it, vi } from 'vitest'
const mocks=vi.hoisted(()=>({getResult:vi.fn()}))
vi.mock('../services/native',()=>({getPlacementResult:mocks.getResult}))
import { PlacementResultPage } from './PlacementResultPage'
afterEach(()=>{cleanup();vi.clearAllMocks()})
const result={attempt:{id:'attempt-1',status:'completed',completedAt:'2026-08-21T10:30:00Z'},estimatedCefrLevel:'B1',confidence:'medium',domains:[{skill:'grammar',level:'B2',assessed:true},{skill:'vocabulary',level:'B1',assessed:true},{skill:'reading',level:'B2',assessed:true},{skill:'spoken_production',level:null,assessed:false}],speakingEvidence:[],speakingSummary:null,listeningAssessed:false,pronunciationAssessed:false,writingAssessed:false,disclaimer:'This is an internal CEFR-informed estimate. It is not an official CEFR certification.'}
function view(){return render(<MemoryRouter initialEntries={['/placement/results/attempt-1']}><Routes><Route path="/placement/results/:attemptId" element={<PlacementResultPage/>}/></Routes></MemoryRouter>)}
describe('PlacementResultPage',()=>{it('shows loading then populated domains, confidence and unassessed skills',async()=>{mocks.getResult.mockResolvedValue(result);view();expect(screen.getByText(/Loading placement result/)).toBeInTheDocument();expect((await screen.findAllByText('B1')).length).toBeGreaterThan(0);expect(screen.getByText('Medium confidence')).toBeInTheDocument();expect(screen.getAllByText('Not assessed').length).toBeGreaterThanOrEqual(4);expect(screen.getByText(/not an official CEFR certification/)).toBeInTheDocument()});it('handles an invalid attempt',async()=>{mocks.getResult.mockResolvedValue(null);view();expect(await screen.findByText('Completed placement result not found.')).toBeInTheDocument()})})
