import { useCallback, useEffect, useState } from 'react'
import type { BackupStatus, BackupSummary, RestoreScheduled } from '../types'
import { createAppBackup, getBackupStatus, listAppBackups, openBackupFolder, restoreAppBackup } from '../services/native'

export function useDataSafety(){
  const [status,setStatus]=useState<BackupStatus|null>(null)
  const [backups,setBackups]=useState<BackupSummary[]>([])
  const [loading,setLoading]=useState(true)
  const [error,setError]=useState<string|null>(null)
  const [restoreResult,setRestoreResult]=useState<RestoreScheduled|null>(null)
  const reload=useCallback(async()=>{setLoading(true);setError(null);try{const[nextStatus,nextBackups]=await Promise.all([getBackupStatus(),listAppBackups()]);setStatus(nextStatus);setBackups(nextBackups)}catch(reason){setError(reason instanceof Error?reason.message:String(reason))}finally{setLoading(false)}},[])
  useEffect(()=>{void reload()},[reload])
  const create=useCallback(async()=>{setError(null);setStatus(value=>value?{...value,operation:'creating'}:value);try{await createAppBackup();await reload()}catch(reason){setError(reason instanceof Error?reason.message:String(reason));setStatus(value=>value?{...value,operation:'failed'}:value)}},[reload])
  const restore=useCallback(async(backupId:string)=>{setError(null);setStatus(value=>value?{...value,operation:'validating'}:value);try{const result=await restoreAppBackup(backupId);setRestoreResult(result);await reload()}catch(reason){setError(reason instanceof Error?reason.message:String(reason));setStatus(value=>value?{...value,operation:'failed'}:value)}},[reload])
  const openFolder=useCallback(async()=>{setError(null);try{await openBackupFolder()}catch(reason){setError(reason instanceof Error?reason.message:String(reason))}},[])
  return{status,backups,loading,error,restoreResult,reload,create,restore,openFolder}
}
