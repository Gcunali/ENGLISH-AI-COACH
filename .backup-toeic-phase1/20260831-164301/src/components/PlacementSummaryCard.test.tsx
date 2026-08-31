// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest'
import { cleanup, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { afterEach, describe, expect, it, vi } from 'vitest'
const mocks=vi.hoisted(()=>({useOverview:vi.fn()}))
vi.mock('../hooks/usePlacementOverview',()=>({usePlacementOverview:mocks.useOverview}))
import { PlacementSummaryCard } from './PlacementSummaryCard'
afterEach(cleanup)
describe('PlacementSummaryCard',()=>{it('offers placement when no result exists',()=>{mocks.useOverview.mockReturnValue({data:{activeAttempt:null,currentResult:null,attemptCount:0},loading:false,error:null});render(<MemoryRouter><PlacementSummaryCard/></MemoryRouter>);expect(screen.getByRole('link',{name:'Take Placement Test'})).toHaveAttribute('href','/placement')});it('shows only the separate persisted placement estimate',()=>{mocks.useOverview.mockReturnValue({data:{activeAttempt:null,attemptCount:1,currentResult:{estimatedCefrLevel:'B2',confidence:'high',attempt:{id:'p1',completedAt:'2026-08-21T10:00:00Z'}}},loading:false,error:null});render(<MemoryRouter><PlacementSummaryCard/></MemoryRouter>);expect(screen.getByText('B2')).toBeInTheDocument();expect(screen.getByText(/Confidence: High/)).toBeInTheDocument();expect(screen.getByRole('link',{name:'View result'})).toHaveAttribute('href','/placement/results/p1')})})
