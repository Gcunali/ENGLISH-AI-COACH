import { useCallback, useEffect, useState } from 'react'
import { getReviewOverview, getReviewSession } from '../services/native'
import type { ReviewOverview, ReviewSession } from '../types'
import { REVIEW_DATA_CHANGED_EVENT } from '../utils/reviewData'

function useQuery<T>(key:string,loader:()=>Promise<T>){const[data,setData]=useState<T|null>(null);const[loading,setLoading]=useState(true);const[error,setError]=useState<string|null>(null);const[revision,setRevision]=useState(0);const reload=useCallback(()=>setRevision(v=>v+1),[]);useEffect(()=>{let disposed=false;setLoading(true);setError(null);void loader().then(v=>{if(!disposed)setData(v)}).catch(e=>{if(!disposed)setError(e instanceof Error?e.message:String(e))}).finally(()=>{if(!disposed)setLoading(false)});return()=>{disposed=true}
  // The stable key intentionally owns loader inputs.
  // eslint-disable-next-line react-hooks/exhaustive-deps
 },[key,revision]);useEffect(()=>{window.addEventListener(REVIEW_DATA_CHANGED_EVENT,reload);return()=>window.removeEventListener(REVIEW_DATA_CHANGED_EVENT,reload)},[reload]);return{data,loading,error,reload}}
export function useReviewOverview(){return useQuery<ReviewOverview>('review-overview',getReviewOverview)}
export function useReviewSession(sessionId:string){return useQuery<ReviewSession|null>(`review-session:${sessionId}`,()=>getReviewSession(sessionId))}
