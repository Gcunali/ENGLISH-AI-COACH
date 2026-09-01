import { CheckCircle2,Headphones,Play,RotateCcw,ShieldCheck } from 'lucide-react'
import { useEffect,useState } from 'react'
import { useNavigate,useParams } from 'react-router-dom'
import { ErrorState,LoadingState } from '../components/PageState'
import { AppCard,InlineNotice,PageHeader,PageShell } from '../components/ProductUI'
import { getToeicFullListening,startToeicFullListening } from '../services/native'
import type { ToeicFullSession } from '../types'

export function ToeicFullListeningPage(){
 const{id}=useParams();const nav=useNavigate();const[data,setData]=useState<ToeicFullSession|null>(null);const[error,setError]=useState<string|null>(null);const[busy,setBusy]=useState(false)
 const load=()=>{if(!id)return;setError(null);void getToeicFullListening(id).then(setData).catch(x=>setError(message(x)))}
 useEffect(load,[id])
 const start=async(mode:'simulation'|'learning')=>{setBusy(true);setError(null);try{const s=await startToeicFullListening(mode);setData(s);nav(`/toeic/listening/${s.sessionId}`,{replace:true})}catch(x){setError(message(x))}finally{setBusy(false)}}
 if(id&&error&&!data)return <ErrorState message={error} onRetry={load}/>
 if(id&&!data)return <LoadingState label="Loading your full Listening simulation…"/>
 if(!data)return <PageShell width="standard"><PageHeader eyebrow="100 questions · Parts 1–4" title="Full Listening Simulation" description="One deterministic offline sequence with resumable progress and an unofficial estimate after all 100 answers."/><InlineNotice tone="info"><ShieldCheck/>Simulation mode hides the final estimate until the complete Listening sequence is finished.</InlineNotice>{error&&<InlineNotice tone="warning">{error}</InlineNotice>}<div className="toeic-results-grid"><AppCard><Headphones/><h2>Simulation mode</h2><p>Complete all four parts in official order. The estimate appears only at 100/100.</p><button className="button-primary" disabled={busy} onClick={()=>void start('simulation')}>Start simulation</button></AppCard><AppCard><RotateCcw/><h2>Learning mode</h2><p>Use the same complete form with the regular feedback available between parts.</p><button className="button-secondary" disabled={busy} onClick={()=>void start('learning')}>Start learning mode</button></AppCard></div></PageShell>
 const current=data.parts.find(p=>p.partNumber===data.currentPart)
 return <PageShell width="standard"><PageHeader eyebrow={`${data.answeredCount} / 100 answered`} title={data.status==='completed'?'Listening complete':'Full Listening Simulation'} description={`Mode: ${data.mode}. Progress is stored locally and can be resumed.`}/>{data.estimate?<section className="toeic-result-hero"><div><p className="eyebrow">Unofficial estimated TOEIC Listening score</p><h1>{data.estimate.estimatedScore}</h1><strong>Estimated range {data.estimate.rangeLow}–{data.estimate.rangeHigh}</strong></div><div className="toeic-score-ring"><span>{data.estimate.rawCorrect}/100</span></div></section>:<InlineNotice tone="info"><Headphones/>No scaled estimate is shown before all 100 questions are complete.</InlineNotice>}<AppCard><div className="toeic-history-list">{data.parts.map(p=><button className="toeic-history-row" key={p.partNumber} disabled={p.partNumber>data.currentPart} onClick={()=>nav(p.route)}><span className="toeic-part-icon">{p.status==='completed'?<CheckCircle2/>:<Headphones/>}</span><span><strong>Part {p.partNumber} — {p.title}</strong><small>{p.questionCount} questions · {p.status}</small></span>{p.partNumber===data.currentPart&&data.status!=='completed'?<Play/>:null}</button>)}</div>{current&&data.status!=='completed'&&<button className="button-primary mt-4" onClick={()=>nav(current.route)}><Play/>Continue Part {current.partNumber}</button>}</AppCard><p className="toeic-disclaimer">{data.disclaimer}</p></PageShell>
}
function message(x:unknown){return x instanceof Error?x.message:String(x)}
