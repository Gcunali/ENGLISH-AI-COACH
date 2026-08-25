// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
const native=vi.hoisted(()=>({getSystemDiagnostics:vi.fn(),listRecentSystemEvents:vi.fn(),exportDiagnosticReport:vi.fn()}))
vi.mock('../services/native',()=>native)
import { DiagnosticsPage } from './DiagnosticsPage'
const healthy={status:'healthy',version:'1',message:'Ready.',technicalCode:null,advancedDetails:{available:true}}
const report={reportVersion:1,generatedAt:'now',appVersion:'0.1.0',platform:'windows x86_64',database:healthy,ollama:healthy,whisper:healthy,piper:healthy,voiceBridge:healthy,voiceStreaming:healthy,pronunciation:healthy,settings:healthy,overallStatus:'All systems ready',conversationReady:true,pronunciationReady:true,databaseReady:true}
beforeEach(()=>{native.getSystemDiagnostics.mockResolvedValue(report);native.listRecentSystemEvents.mockResolvedValue([]);native.exportDiagnosticReport.mockResolvedValue(JSON.stringify(report));Object.assign(navigator,{clipboard:{writeText:vi.fn().mockResolvedValue(undefined)}})})
afterEach(()=>{cleanup();vi.clearAllMocks()})
describe('DiagnosticsPage',()=>{
  it('shows loading and then all component cards',async()=>{native.getSystemDiagnostics.mockReturnValueOnce(new Promise(()=>undefined));render(<DiagnosticsPage/>);expect(screen.getByText('Checking local componentsâ€¦')).toBeInTheDocument();cleanup();native.getSystemDiagnostics.mockResolvedValue(report);render(<DiagnosticsPage/>);expect(await screen.findByText('All systems ready')).toBeInTheDocument();expect(screen.getByRole('heading',{name:'Pronunciation'})).toBeInTheDocument()})
  it('keeps component readiness independent',async()=>{native.getSystemDiagnostics.mockResolvedValue({...report,pronunciation:{...healthy,status:'unavailable',message:'Pronunciation unavailable.'},pronunciationReady:false,overallStatus:'Some components need attention'});render(<DiagnosticsPage/>);expect(await screen.findByText('Some components need attention')).toBeInTheDocument();expect(screen.getByText('Conversation').parentElement).toHaveTextContent('Ready');expect(screen.getByText('Pronunciation unavailable.')).toBeInTheDocument()})
  it('reruns diagnostics and copies the sanitized report',async()=>{render(<DiagnosticsPage/>);await screen.findByText('All systems ready');fireEvent.click(screen.getByRole('button',{name:/Run Diagnostics Again/}));await waitFor(()=>expect(native.getSystemDiagnostics).toHaveBeenCalledTimes(2));fireEvent.click(screen.getByRole('button',{name:/Copy Diagnostic Report/}));await waitFor(()=>expect(navigator.clipboard.writeText).toHaveBeenCalled())})
  it('shows database warning and advanced details without HTML injection',async()=>{native.getSystemDiagnostics.mockResolvedValue({...report,database:{...healthy,status:'warning',message:'Foreign keys need attention.',advancedDetails:{foreignKeyViolations:1}},databaseReady:false,overallStatus:'Core conversation unavailable'});render(<DiagnosticsPage/>);expect(await screen.findByText('Foreign keys need attention.')).toBeInTheDocument();fireEvent.click(screen.getAllByText('Advanced details')[0]);expect(screen.getByText(/foreignKeyViolations/)).toBeInTheDocument()})
})
