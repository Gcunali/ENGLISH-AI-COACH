import { ArrowRight, ClipboardCheck, MessageCircle, ShieldCheck, X } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { getWelcomeState, setWelcomeSeen } from '../services/native'

export function WelcomePanel() {
  const [visible, setVisible] = useState(false)
  const [saving, setSaving] = useState(false)
  useEffect(() => { let disposed = false; void getWelcomeState().then((value) => { if (!disposed) setVisible(value.shouldShow) }).catch((error) => console.warn('Welcome state unavailable', error)); return () => { disposed = true } }, [])
  const dismiss = async () => { if (saving) return; setSaving(true); try { await setWelcomeSeen(true); setVisible(false) } finally { setSaving(false) } }
  if (!visible) return null
  return <section className="welcome-panel" aria-labelledby="welcome-title"><div className="welcome-copy"><div className="welcome-icon"><ShieldCheck aria-hidden="true" size={22} /></div><div><p className="eyebrow">Local · Private · Offline capable</p><h2 id="welcome-title" className="section-title text-xl">Welcome to your English practice space.</h2><p className="page-description">Talk with a local AI teacher, estimate your level, and follow your progress—all from this computer.</p><div className="mt-5 flex flex-wrap gap-3"><Link to="/lesson/new" onClick={() => void dismiss()} className="button-primary"><MessageCircle size={16} />Start a Conversation</Link><Link to="/placement" onClick={() => void dismiss()} className="button-secondary"><ClipboardCheck size={16} />Take Placement Test</Link><button type="button" onClick={() => void dismiss()} className="button-ghost">Explore Home <ArrowRight size={15} /></button></div></div></div><button type="button" disabled={saving} onClick={() => void dismiss()} className="icon-button" aria-label="Dismiss welcome"><X size={18} /></button></section>
}
