import { describe, expect, it } from 'vitest'
import type { PronunciationAttempt } from '../types'
import { WORD_PRONUNCIATION_FEEDBACK_VERSION, wordFeedback } from './pronunciationFeedback'

const attempt = (confidence: 'low'|'medium'|'high', coverage: number): PronunciationAttempt => ({ id:'a', status:'completed', sourceType:'custom', sourceId:null, targetText:'hello world', locale:'en-US', overallScore:75, confidence, contentMatchScore:1, alignmentCoverage:coverage, audioDurationMs:500, createdAt:'x', completedAt:'x', words:[{index:0,word:'hello',score:90,startMs:0,endMs:200,expectedPhones:['h'],phoneResults:[]},{index:1,word:'world',score:60,startMs:200,endMs:500,expectedPhones:['w'],phoneResults:[]}] })
describe('word pronunciation feedback v1', () => {
  it('hides specific word claims when alignment confidence is low', () => expect(wordFeedback(attempt('low', 1)).available).toBe(false))
  it('labels reliable acoustic evidence and selects a focus word', () => { const result=wordFeedback(attempt('high', .95)); expect(WORD_PRONUNCIATION_FEEDBACK_VERSION).toBe(1); expect(result.items.map(x=>x.label)).toEqual(['Strong','Needs attention']); expect(result.items[1].focus).toBe(true) })
})
