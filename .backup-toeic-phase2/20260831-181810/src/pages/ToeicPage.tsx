import { BookOpenCheck, Check, Clock3, Headphones, History, LockKeyhole, Play } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { ErrorState, LoadingState } from '../components/PageState'
import { AppCard, InlineNotice, PageHero, PageShell, SectionHeader } from '../components/ProductUI'
import { getToeicOverview, startToeicSession } from '../services/native'
import type { ToeicOverview } from '../types'

export function ToeicPage() {
  const navigate = useNavigate(); const [data,setData]=useState<ToeicOverview|null>(null); const [error,setError]=useState<string|null>(null); const [busy,setBusy]=useState<string|null>(null)
  const load=()=>{setError(null);void getToeicOverview().then(setData).catch(value=>setError(message(value)))}
  useEffect(load,[])
  if(error&&!data)return <ErrorState message={error} onRetry={load}/>
  if(!data)return <LoadingState label="Loading the local TOEIC item bank…"/>
  const start=async(formId:string,version:number)=>{setBusy(formId);setError(null);try{const session=await startToeicSession(formId,version);navigate(`/toeic/session/${session.sessionId}`)}catch(value){setError(message(value))}finally{setBusy(null)}}
  return <PageShell width="wide">
    <PageHero eyebrow="Exam preparation" title="TOEIC Preparation" accent="Listening & Reading" description="Focused, offline TOEIC-style practice with deterministic grading and authored feedback. Part 1 is available now." compact />
    {error&&<InlineNotice tone="warning" live>{error}</InlineNotice>}
    <div className="toeic-trust-row"><span><Clock3/>Untimed by design</span><span><LockKeyhole/>Answers stored locally</span><span><BookOpenCheck/>Authored explanations</span></div>
    {data.activeSessions.length>0&&<AppCard className="mt-4 toeic-resume-card"><SectionHeader title="Resume TOEIC Practice" description="Continue exactly where you stopped—there is no pause penalty."/><div className="toeic-history-list">{data.activeSessions.map(entry=><button key={entry.sessionId} className="toeic-history-row" onClick={()=>navigate(`/toeic/session/${entry.sessionId}`)}><Play aria-hidden="true"/><span><strong>{entry.formTitle}</strong><small>{entry.answered} of 6 answered</small></span><b>Resume</b></button>)}</div></AppCard>}
    <AppCard className="mt-4">
      <SectionHeader title="Listening Part 1 — Photographs" description="Look at each photograph, listen once to four statements, and choose the best description." />
      <div className="toeic-form-grid">{data.forms.map(form=><article className="toeic-form-card" key={form.formId}><div className="toeic-part-icon"><Headphones/></div><div><p className="eyebrow">6 questions · Untimed</p><h3>{form.title}</h3><p>Original local photographs and four audio statements per question.</p></div><button className="button-primary" disabled={busy!==null} onClick={()=>form.activeSessionId?navigate(`/toeic/session/${form.activeSessionId}`):void start(form.formId,form.formVersion)}>{form.activeSessionId?'Resume':busy===form.formId?'Starting…':'Start form'}</button></article>)}</div>
    </AppCard>
    <AppCard className="mt-4"><div className="section-header"><div><p className="eyebrow">Full exam architecture</p><h2 className="section-title">Listening & Reading · 200 questions</h2><p className="section-description">Only Part 1 is enabled. The remaining parts are shown honestly as unavailable.</p></div><button className="button-secondary" onClick={()=>navigate('/toeic/history')}><History/>TOEIC Performance</button></div><div className="toeic-parts-grid">{data.parts.map((part,index)=><div className={`toeic-part-row ${part.runtimeAvailable?'available':''}`} key={part.part}><span>{part.runtimeAvailable?<Check/>:<LockKeyhole/>}</span><div><strong>Part {index+1} — {part.title}</strong><small>{part.questionCount} questions</small></div><b>{part.runtimeAvailable?'Available':'Not yet available'}</b></div>)}</div></AppCard>
    <p className="toeic-disclaimer">{data.disclaimer}</p>
  </PageShell>
}
function message(value:unknown){return value instanceof Error?value.message:String(value)}
