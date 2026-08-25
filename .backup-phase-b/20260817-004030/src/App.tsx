import { useCallback, useEffect, useMemo, useState } from 'react'
import { Activity, BookOpen, ChevronRight, CircleGauge, Clock3, Database, Gauge, History, Mic, Mic2, Pause, Play, Settings, ShieldCheck, Sparkles, Square, WifiOff } from 'lucide-react'
import { getDiagnostics, listOllamaModels } from './services/native'
import { useVoicePipeline } from './hooks/useVoicePipeline'
import { useConversationStore } from './stores/conversation'
import type { Diagnostics, OllamaModel } from './types'

const EMPTY_DIAGNOSTICS: Diagnostics = { components: [], dataDirectory: '', offlineReady: false, platform: 'Windows' }
const STATE_LABELS = {
  IDLE: 'Ready when you are', PREPARING: 'Preparing your teacher…', LISTENING: 'Listening…',
  STUDENT_SPEAKING: 'You are speaking…', TRANSCRIBING: 'Transcribing locally…',
  TEACHER_THINKING: 'Teacher is thinking…', TEACHER_SPEAKING: 'Teacher is speaking…',
  PAUSED: 'Lesson paused', ENDING: 'Saving lesson…', ANALYZING: 'Analyzing locally…',
  COMPLETED: 'Lesson complete', ERROR: 'Local component unavailable',
} as const

function App() {
  const [diagnostics, setDiagnostics] = useState(EMPTY_DIAGNOSTICS)
  const [models, setModels] = useState<OllamaModel[]>([])
  const [model, setModel] = useState('qwen3.5:4b')
  const [showTranscript, setShowTranscript] = useState(true)
  const [developerMode, setDeveloperMode] = useState(false)
  const state = useConversationStore((store) => store.state)
  const messages = useConversationStore((store) => store.messages)
  const metrics = useConversationStore((store) => store.metrics)
  const error = useConversationStore((store) => store.error)
  const voice = useVoicePipeline(model)

  const refresh = useCallback(async () => {
    const [health, installedModels] = await Promise.all([getDiagnostics(), listOllamaModels().catch(() => [])])
    setDiagnostics(health)
    setModels(installedModels)
    if (installedModels.length && !installedModels.some((item) => item.name === model)) setModel(installedModels[0].name)
  }, [model])
  useEffect(() => { void refresh() }, [refresh])

  const readyCount = diagnostics.components.filter((component) => component.ready).length
  const active = !['IDLE', 'COMPLETED', 'ERROR'].includes(state)
  const waveform = useMemo(() => Array.from({ length: 28 }, (_, index) => 12 + ((index * 19) % 42)), [])

  return (
    <div className="app-shell flex min-h-screen">
      <aside className="desktop-nav w-64 shrink-0 border-r border-white/[.07] p-5 flex flex-col">
        <div className="flex items-center gap-3 px-2 py-2">
          <div className="h-9 w-9 rounded-xl bg-[var(--accent)] text-black grid place-items-center"><Sparkles size={18} /></div>
          <div><div className="font-semibold tracking-tight">English AI</div><div className="text-xs muted">Local Coach</div></div>
        </div>
        <nav className="mt-10 space-y-1 text-sm" aria-label="Main navigation">
          <NavItem icon={<Mic2 size={17} />} label="Conversation" active />
          <NavItem icon={<CircleGauge size={17} />} label="Progress" />
          <NavItem icon={<History size={17} />} label="History" />
          <NavItem icon={<BookOpen size={17} />} label="Vocabulary" />
          <NavItem icon={<Settings size={17} />} label="Settings" />
        </nav>
        <div className="mt-auto glass rounded-2xl p-4">
          <div className="flex items-center gap-2 text-sm font-medium"><ShieldCheck size={16} className="text-[var(--accent)]" /> Private by default</div>
          <p className="muted text-xs leading-5 mb-0">Audio, transcripts and learning data stay on this computer.</p>
        </div>
      </aside>

      <main className="flex-1 min-w-0 p-5 md:p-8 max-w-[1500px] mx-auto">
        <header className="flex flex-wrap items-center justify-between gap-4 mb-7">
          <div><p className="muted text-xs uppercase tracking-[.18em] mb-2">Conversation lab</p><h1 className="text-2xl md:text-3xl font-semibold tracking-tight m-0">Speak more. Think less.</h1></div>
          <div className="flex items-center gap-2 rounded-full border border-white/10 px-3 py-2 text-xs"><WifiOff size={14} className="text-[var(--accent)]" /> No cloud required</div>
        </header>

        <div className="content-grid grid grid-cols-[minmax(0,1fr)_330px] gap-5">
          <section className="glass rounded-[28px] min-h-[620px] p-5 md:p-8 flex flex-col overflow-hidden relative" aria-label="Conversation workspace">
            {!diagnostics.offlineReady && <div className="absolute top-5 right-5 text-[10px] font-semibold tracking-widest bg-amber-400/10 text-amber-200 px-3 py-1.5 rounded-full border border-amber-300/10">SETUP REQUIRED</div>}
            <div className="flex items-center gap-3 text-sm">
              <div className="rounded-full bg-white/5 px-3 py-1.5">Free conversation</div>
              <div className="muted flex items-center gap-1.5"><Clock3 size={14} /> 20 min</div>
            </div>

            <div className="flex-1 flex flex-col items-center justify-center py-10">
              <div className="relative h-44 w-44 grid place-items-center">
                {active && <div className="pulse-ring absolute inset-2 rounded-full border border-[var(--accent)]/40" />}
                <div className="absolute inset-5 rounded-full bg-gradient-to-br from-[#1d2634] to-[#0c1017] border border-white/10 shadow-2xl" />
                <div className="relative flex items-center gap-[3px] h-16" aria-hidden="true">
                  {waveform.map((height, index) => <i key={index} className="bar block w-[3px] rounded-full bg-[var(--accent)]" style={{ height: `${Math.max(8, height * (state === 'STUDENT_SPEAKING' ? .5 + voice.level : .35))}px`, animationDelay: `${index * 35}ms` }} />)}
                </div>
              </div>
              <p className="mt-6 mb-1 text-sm font-semibold tracking-[.12em] uppercase">{STATE_LABELS[state]}</p>
              <p className="muted text-sm text-center max-w-md">{error ?? (diagnostics.offlineReady ? 'Everything is processed on this device.' : 'Install the missing local components shown at right, then run diagnostics again.')}</p>

              <div className="mt-7 flex flex-wrap justify-center gap-2">
                {!active ? (
                  <button onClick={() => void voice.startLesson()} className="bg-[var(--accent)] text-[#081006] font-semibold rounded-full px-6 py-3 flex items-center gap-2" aria-label="Start conversation"><Play size={17} fill="currentColor" /> Start conversation</button>
                ) : (
                  <>
                    <button className="h-11 w-11 rounded-full border border-white/10 grid place-items-center" aria-label="Pause lesson"><Pause size={17} /></button>
                    {voice.mode === 'PUSH_TO_TALK' && <button onPointerDown={voice.beginPushToTalk} onPointerUp={voice.endPushToTalk} className="bg-[var(--accent)] text-black rounded-full px-5 py-2 font-semibold flex items-center gap-2"><Mic size={17} /> Hold to speak</button>}
                    <button onClick={voice.endLesson} className="h-11 px-4 rounded-full border border-red-300/20 text-red-200 flex items-center gap-2" aria-label="End lesson"><Square size={14} fill="currentColor" /> End</button>
                  </>
                )}
              </div>
              <div className="mt-4 flex rounded-full bg-black/25 p-1 text-xs">
                {(['AUTO', 'PUSH_TO_TALK'] as const).map((item) => <button key={item} onClick={() => voice.setMode(item)} className={`px-3 py-1.5 rounded-full ${voice.mode === item ? 'bg-white/10 text-white' : 'muted'}`}>{item === 'AUTO' ? 'Auto VAD' : 'Push to talk'}</button>)}
              </div>
            </div>

            <div className="border-t border-white/[.07] pt-4">
              <button onClick={() => setShowTranscript((value) => !value)} className="flex w-full items-center justify-between text-sm bg-transparent border-0 text-white p-0"><span>Live transcript <span className="muted ml-2">{showTranscript ? 'On' : 'Off'}</span></span><ChevronRight size={16} className={showTranscript ? 'rotate-90' : ''} /></button>
              {showTranscript && <div className="mt-4 max-h-40 overflow-auto scrollbar-none space-y-3" aria-live="polite">
                {messages.length === 0 ? <p className="muted text-sm">Your conversation will appear here. Audio is deleted after local transcription.</p> : messages.map((message) => <div key={message.id} className="grid grid-cols-[64px_1fr] gap-3 text-sm"><span className={message.role === 'teacher' ? 'text-[var(--accent)] text-xs font-semibold pt-0.5' : 'text-sky-300 text-xs font-semibold pt-0.5'}>{message.role === 'teacher' ? 'TEACHER' : 'YOU'}</span><span>{message.text}</span></div>)}
              </div>}
            </div>
          </section>

          <aside className="space-y-5">
            <section className="glass rounded-[22px] p-5" aria-labelledby="status-heading">
              <div className="flex items-center justify-between"><div><p className="muted text-[10px] tracking-widest uppercase mb-1">Local stack</p><h2 id="status-heading" className="text-base m-0">System status</h2></div><button onClick={() => void refresh()} className="bg-white/5 border border-white/10 rounded-lg p-2 text-white" aria-label="Refresh diagnostics"><Activity size={16} /></button></div>
              <div className="mt-5 space-y-3.5">
                {diagnostics.components.map((component) => <div key={component.name} className="flex items-start gap-3"><span className={`status-dot mt-1.5 ${component.ready ? 'bg-[var(--accent)] shadow-[0_0_10px_var(--accent)]' : 'bg-amber-400'}`} /><div className="min-w-0"><div className="text-sm">{component.label}</div><div className="muted text-[11px] truncate" title={component.detail}>{component.detail}</div></div></div>)}
                {!diagnostics.components.length && <p className="muted text-sm">Running diagnostics…</p>}
              </div>
              <div className="mt-5 pt-4 border-t border-white/[.07] flex justify-between text-xs"><span className="muted">Components ready</span><span>{readyCount}/{diagnostics.components.length || 8}</span></div>
            </section>

            <section className="glass rounded-[22px] p-5">
              <label className="text-xs muted block mb-2" htmlFor="model">Conversation model</label>
              <select id="model" value={model} onChange={(event) => setModel(event.target.value)} className="w-full rounded-xl bg-[#0d1119] border border-white/10 px-3 py-2.5 text-sm text-white">
                {models.length ? models.map((item) => <option key={item.name} value={item.name}>{item.name}</option>) : <><option>qwen3.5:4b</option><option>llama3.2:3b</option></>}
              </select>
              <div className="mt-4 grid grid-cols-2 gap-2">
                <MiniStat icon={<Mic size={14} />} label="Mic level" value={`${Math.round(voice.level * 100)}%`} />
                <MiniStat icon={<Database size={14} />} label="Storage" value="Local" />
              </div>
            </section>

            <section className="glass rounded-[22px] p-5">
              <button onClick={() => setDeveloperMode((value) => !value)} className="w-full flex items-center justify-between bg-transparent border-0 p-0 text-white"><span className="flex items-center gap-2 text-sm"><Gauge size={16} /> Performance</span><span className="muted text-xs">{developerMode ? 'Hide' : 'Show'}</span></button>
              {developerMode && <div className="mt-4 grid grid-cols-2 gap-y-3 text-xs"><Metric label="STT" value={metrics ? `${metrics.sttMs} ms` : '—'} /><Metric label="LLM" value={metrics ? `${metrics.llmMs} ms` : '—'} /><Metric label="TTS" value={metrics ? `${metrics.ttsMs} ms` : '—'} /><Metric label="Total" value={metrics ? `${metrics.totalMs} ms` : '—'} /></div>}
            </section>
          </aside>
        </div>
      </main>
    </div>
  )
}

function NavItem({ icon, label, active = false }: { icon: React.ReactNode; label: string; active?: boolean }) {
  return <button className={`w-full flex items-center gap-3 rounded-xl px-3 py-2.5 border-0 text-left ${active ? 'bg-white/[.07] text-white' : 'bg-transparent muted'}`}>{icon}<span>{label}</span></button>
}
function MiniStat({ icon, label, value }: { icon: React.ReactNode; label: string; value: string }) {
  return <div className="rounded-xl bg-white/[.035] p-3"><div className="muted flex items-center gap-1.5 text-[10px]">{icon}{label}</div><div className="mt-1 text-sm">{value}</div></div>
}
function Metric({ label, value }: { label: string; value: string }) { return <div><div className="muted">{label}</div><div className="mt-1">{value}</div></div> }

export default App
