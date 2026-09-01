import { DatabaseBackup, ExternalLink, FolderOpen, RotateCcw } from 'lucide-react'
import { useState } from 'react'
import { Link } from 'react-router-dom'
import { useDataSafety } from '../hooks/useDataSafety'
import { ConfirmationDialog } from './ConfirmationDialog'

export function DataBackupSection(){
  const data=useDataSafety(); const [confirming,setConfirming]=useState<string|null>(null)
  const busy=!!data.status&&!['idle','completed','failed'].includes(data.status.operation)
  const selected=data.backups.find(item=>item.backupId===confirming)
  return <section className="glass mt-5 rounded-[28px] p-5 md:p-7" aria-labelledby="data-backup-title">
    <div className="flex flex-wrap items-start justify-between gap-4"><div className="flex min-w-0 gap-3"><DatabaseBackup className="mt-1 shrink-0 text-[var(--accent)]" size={21}/><div><p className="eyebrow">Data &amp; Backup</p><h2 id="data-backup-title" className="section-title">Protect your learning data</h2><p className="section-description">Create a validated local snapshot of lessons, progress and settings.</p></div></div><Link to="/diagnostics" className="button-secondary"><ExternalLink size={16}/>System Diagnostics</Link></div>
    <div className="mt-5 flex flex-wrap gap-3"><button type="button" aria-busy={busy} disabled={busy||data.loading} onClick={()=>void data.create()} className="button-primary">{data.status?.operation==='creating'||data.status?.operation==='validating'?'Creating backup…':'Create Backup'}</button><button type="button" onClick={()=>void data.openFolder()} className="button-secondary"><FolderOpen size={16}/>Open Backup Folder</button></div>
    {data.error&&<p role="alert" className="mt-4 text-sm text-red-300">{data.error}</p>}
    {data.status?.pendingRestart&&<p role="status" className="mt-4 rounded-xl border border-amber-300/20 bg-amber-300/[.06] p-3 text-sm text-amber-100">Restore is ready. Restart the app to finish safely.</p>}
    {data.restoreResult&&<p role="status" className="mt-4 text-sm text-emerald-200">{data.restoreResult.message}</p>}
    <div className="mt-5 rounded-2xl bg-white/[.03] p-4"><h3 className="m-0 text-sm">Available backups</h3>{data.loading?<p className="muted mb-0 mt-3 text-sm">Loading backups…</p>:data.backups.length===0?<p className="muted mb-0 mt-3 text-sm">No backups created yet.</p>:<ul className="m-0 mt-3 space-y-3 p-0">{data.backups.map((backup,index)=><li key={backup.backupId} className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-white/[.07] p-3"><div className="min-w-0"><p className="m-0 text-sm font-medium">{index===0?'Last backup':'Backup'} · Schema {backup.schemaVersion}</p><p className="muted mb-0 mt-1 break-all text-xs">{backup.createdAt} · {(backup.databaseBytes/1024).toFixed(1)} KB</p></div><button type="button" disabled={busy||!data.status?.restoreAllowed||data.status?.pendingRestart} aria-label={`Restore backup ${backup.createdAt}`} onClick={()=>setConfirming(backup.backupId)} className="button-secondary"><RotateCcw size={15}/>Restore</button></li>)}</ul>}</div>
    {data.status?.restoreBlockReason&&<p className="muted mt-3 text-xs">{data.status.restoreBlockReason}</p>}
    <p className="muted mb-0 mt-4 text-xs">Backups contain your local learning data and are not encrypted. Keep them in a location you trust. Models, Python environments, audio, caches and logs are excluded.</p>
    <ConfirmationDialog open={!!selected} title="Restore this backup?" description={<><p className="m-0">Restoring replaces your current learning data. A safety backup of the current database will be created first.</p><p className="mb-0 break-all text-xs">Selected: {selected?.createdAt}</p></>} confirmLabel="Restore Backup" danger busy={busy} onClose={()=>setConfirming(null)} onConfirm={()=>{if(!selected)return;setConfirming(null);void data.restore(selected.backupId)}}/>
  </section>
}
