import { useCallback, useEffect, useRef, useState } from 'react'
import { analyzePronunciation, cancelPronunciationAnalysis, getPronunciationEngineStatus, listPronunciationAttempts } from '../services/native'
import type { PronunciationAttempt, PronunciationEngineStatus } from '../types'
import { downsample, encodeWavBase64 } from '../utils/audio'
import { trimPlacementSpeech } from '../utils/placementAudio'

export type PronunciationUiState='loading_engine'|'idle'|'recording'|'checking'|'analyzing'|'completed'|'error'

export function validatePronunciationTarget(value:string):string|null{
  const words=value.match(/[A-Za-z]+(?:['’][A-Za-z]+)?/g)??[]
  if(!words.length)return 'Enter a word or short phrase.'
  if(value.length>160)return 'Use 160 characters or fewer.'
  if(words.length>12)return 'Use 12 words or fewer.'
  return null
}

export function usePronunciationPractice(initialTarget='',sourceType:'custom'|'vocabulary'='custom',sourceId:string|null=null){
  const [target,setTarget]=useState(initialTarget.slice(0,160));const [state,setState]=useState<PronunciationUiState>('loading_engine');const [engine,setEngine]=useState<PronunciationEngineStatus|null>(null);const [result,setResult]=useState<PronunciationAttempt|null>(null);const [history,setHistory]=useState<PronunciationAttempt[]>([]);const [error,setError]=useState<string|null>(null)
  const context=useRef<AudioContext|null>(null),stream=useRef<MediaStream|null>(null),source=useRef<MediaStreamAudioSourceNode|null>(null),worklet=useRef<AudioWorkletNode|null>(null),samples=useRef<Float32Array[]>([]),timer=useRef<number|null>(null),analysisTimer=useRef<number|null>(null)
  const cleanup=useCallback(()=>{if(timer.current!==null)window.clearTimeout(timer.current);if(analysisTimer.current!==null)window.clearTimeout(analysisTimer.current);timer.current=null;analysisTimer.current=null;worklet.current?.disconnect();source.current?.disconnect();stream.current?.getTracks().forEach(track=>track.stop());void context.current?.close();worklet.current=null;source.current=null;stream.current=null;context.current=null},[])
  const refreshHistory=useCallback(async()=>setHistory(await listPronunciationAttempts(20)),[])
  useEffect(()=>{let active=true;void Promise.all([getPronunciationEngineStatus(),listPronunciationAttempts(20)]).then(([status,items])=>{if(!active)return;setEngine(status);setHistory(items);setState(status.ready?'idle':'error');if(!status.ready)setError(status.lastError??'Local pronunciation engine is unavailable.')}).catch(value=>{if(active){setError(value instanceof Error?value.message:String(value));setState('error')}});return()=>{active=false;cleanup();void cancelPronunciationAnalysis()}},[cleanup])
  const stop=useCallback(async()=>{const audioContext=context.current;const chunks=trimPlacementSpeech(samples.current);cleanup();if(!audioContext||!chunks?.length){setError('No valid speech was detected. Please record again.');setState('error');return}const length=chunks.reduce((sum,chunk)=>sum+chunk.length,0),combined=new Float32Array(length);let offset=0;for(const chunk of chunks){combined.set(chunk,offset);offset+=chunk.length}setState('checking');setError(null);analysisTimer.current=window.setTimeout(()=>setState('analyzing'),700);try{const wav=encodeWavBase64(downsample(combined,audioContext.sampleRate),16000);const attempt=await analyzePronunciation(target,wav,sourceType,sourceId);setResult(attempt);setState('completed');await refreshHistory()}catch(value){setError(value instanceof Error?value.message:String(value));setState('error')}finally{if(analysisTimer.current!==null)window.clearTimeout(analysisTimer.current);analysisTimer.current=null}},[cleanup,refreshHistory,sourceId,sourceType,target])
  const start=useCallback(async()=>{const validation=validatePronunciationTarget(target);if(validation){setError(validation);setState('error');return}cleanup();samples.current=[];setResult(null);setError(null);try{const media=await navigator.mediaDevices.getUserMedia({audio:{channelCount:1,echoCancellation:true,noiseSuppression:true,autoGainControl:true}});const audioContext=new AudioContext({latencyHint:'interactive'});await audioContext.audioWorklet.addModule('/pcm-capture-processor.js');const input=audioContext.createMediaStreamSource(media),processor=new AudioWorkletNode(audioContext,'pcm-capture-processor'),silent=audioContext.createGain();silent.gain.value=0;input.connect(processor).connect(silent).connect(audioContext.destination);processor.port.onmessage=(event:MessageEvent<Float32Array>)=>samples.current.push(event.data);context.current=audioContext;stream.current=media;source.current=input;worklet.current=processor;setState('recording');timer.current=window.setTimeout(()=>{void stop()},15000)}catch(value){cleanup();setError(`Microphone unavailable: ${value instanceof Error?value.message:String(value)}`);setState('error')}},[cleanup,stop,target])
  const retry=useCallback(()=>{setResult(null);setError(null);setState(engine?.ready?'idle':'error')},[engine]);const cancel=useCallback(async()=>{cleanup();await cancelPronunciationAnalysis();setResult(null);setState(engine?.ready?'idle':'error')},[cleanup,engine])
  return{target,setTarget,state,engine,result,history,error,start,stop,retry,cancel}
}
