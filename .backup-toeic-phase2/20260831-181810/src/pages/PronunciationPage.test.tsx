// @vitest-environment jsdom

import '@testing-library/jest-dom/vitest'
import { fireEvent, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { PronunciationPage } from './PronunciationPage'
import { validatePronunciationTarget } from '../hooks/usePronunciationPractice'

const mocks=vi.hoisted(()=>({hook:vi.fn()}))
vi.mock('../hooks/usePronunciationPractice',async()=>{const actual=await vi.importActual<typeof import('../hooks/usePronunciationPractice')>('../hooks/usePronunciationPractice');return{...actual,usePronunciationPractice:mocks.hook}})

function base(overrides:Record<string,unknown>={}){return{target:"I'm terrible at cooking.",setTarget:vi.fn(),state:'idle',engine:{ready:true},result:null,history:[],error:null,start:vi.fn(),stop:vi.fn(),retry:vi.fn(),cancel:vi.fn(),...overrides}}
describe('PronunciationPage',()=>{
  beforeEach(()=>mocks.hook.mockReturnValue(base()))
  it('prefills a vocabulary target without rendering HTML',()=>{mocks.hook.mockImplementation((target:string)=>base({target}));render(<MemoryRouter initialEntries={['/pronunciation?text=%3Cimg%20src%3Dx%3E&source=vocabulary&sourceId=v1']}><PronunciationPage/></MemoryRouter>);expect(mocks.hook).toHaveBeenCalledWith('<img src=x>','vocabulary','v1');expect(screen.getByDisplayValue('<img src=x>')).toBeInTheDocument();expect(document.querySelector('img')).toBeNull()})
  it('validates empty and oversized targets',()=>{expect(validatePronunciationTarget('')).toMatch(/Enter/);expect(validatePronunciationTarget('word '.repeat(13))).toMatch(/12 words/);expect(validatePronunciationTarget('think')).toBeNull()})
  it('shows loading and unavailable states',()=>{mocks.hook.mockReturnValue(base({state:'loading_engine'}));const{rerender}=render(<MemoryRouter><PronunciationPage/></MemoryRouter>);expect(screen.getByText(/Loading local/)).toBeInTheDocument();mocks.hook.mockReturnValue(base({state:'error',engine:{ready:false},error:'Model unavailable'}));rerender(<MemoryRouter><PronunciationPage/></MemoryRouter>);expect(screen.getByRole('alert')).toHaveTextContent('Model unavailable')})
  it('renders content mismatch honestly',()=>{mocks.hook.mockReturnValue(base({state:'completed',result:{status:'content_mismatch'}}));render(<MemoryRouter><PronunciationPage/></MemoryRouter>);expect(screen.getByText(/did not match the target/)).toBeInTheDocument()})
  it('expands word and phone feedback by keyboard-accessible button',()=>{mocks.hook.mockReturnValue(base({state:'completed',result:{status:'completed',overallScore:78,confidence:'medium',alignmentCoverage:.95,targetText:'think',words:[{index:0,word:'think',score:78,expectedPhones:['θ'],phoneResults:[{phone:'θ',score:62,closestAlternative:'s',hint:'Use airflow.'}]}]}}));render(<MemoryRouter><PronunciationPage/></MemoryRouter>);fireEvent.click(screen.getByRole('button',{name:/think/}));expect(screen.getByText('/θ/')).toBeInTheDocument();expect(screen.getByText(/Use airflow/)).toBeInTheDocument()})
  it('shows recent attempts without transcript',()=>{mocks.hook.mockReturnValue(base({history:[{id:'a',targetText:'coffee',overallScore:81,confidence:'high',status:'completed',createdAt:'2026-08-23T00:00:00Z'}]}));render(<MemoryRouter><PronunciationPage/></MemoryRouter>);expect(screen.getByText('coffee')).toBeInTheDocument();expect(screen.queryByText(/heard:/i)).not.toBeInTheDocument()})
})
