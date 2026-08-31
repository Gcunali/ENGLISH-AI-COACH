import { Activity, BookOpen, ClipboardCheck, MessageCircle, ShieldCheck, X } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { getWelcomeState, setWelcomeSeen } from '../services/native'

export function WelcomePanel() {
  const [visible, setVisible] = useState(false)
  const [saving, setSaving] = useState(false)
  useEffect(() => { let disposed = false; void getWelcomeState().then((value) => { if (!disposed) setVisible(value.shouldShow) }).catch((error) => console.warn('Welcome state unavailable', error)); return () => { disposed = true } }, [])
  const dismiss = async () => { if (saving) return; setSaving(true); try { await setWelcomeSeen(true); setVisible(false) } finally { setSaving(false) } }
  if (!visible) return null
  return <section className="welcome-panel" aria-labelledby="welcome-title"><div className="welcome-copy"><div className="welcome-icon"><ShieldCheck aria-hidden="true" size={22} /></div><div><p className="eyebrow">Local · Private · Your pace</p><h2 id="welcome-title" className="section-title text-xl">Welcome to your English learning space.</h2><p className="page-description">The AI runs locally, learning data stays on this computer, and audio is not saved by default. You control when to create or restore a backup.</p><ol className="mt-4 grid gap-2 pl-5 text-sm"><li><strong>Check readiness.</strong> Diagnostics shows whether local voice components are available.</li><li><strong>Choose a starting point.</strong> Take the optional Placement Test or pick any Course level.</li><li><strong>Follow a Guided Lesson.</strong> Theory, vocabulary, listening, speaking, exercises, conversation and analysis keep the next action clear.</li><li><strong>Practice freely.</strong> Start a local conversation whenever you want.</li></ol><div className="mt-5 flex flex-wrap gap-3"><Link to="/course" onClick={() => void dismiss()} className="button-primary"><BookOpen size={16} />Open Course</Link><Link to="/placement" onClick={() => void dismiss()} className="button-secondary"><ClipboardCheck size={16} />Optional Placement Test</Link><Link to="/lesson/new" onClick={() => void dismiss()} className="button-secondary"><MessageCircle size={16} />Free Conversation</Link><Link to="/diagnostics" className="button-ghost"><Activity size={15} />Check Readiness</Link></div></div></div><button type="button" disabled={saving} onClick={() => void dismiss()} className="icon-button" aria-label="Dismiss welcome"><X size={18} /></button></section>
}
