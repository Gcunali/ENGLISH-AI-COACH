import { describe,expect,it } from 'vitest'
import { validatePronunciationTarget } from './usePronunciationPractice'
describe('validatePronunciationTarget',()=>{it('accepts contractions and punctuation',()=>expect(validatePronunciationTarget("I'm ready.")).toBeNull());it('enforces character and word bounds',()=>{expect(validatePronunciationTarget('')).not.toBeNull();expect(validatePronunciationTarget('x'.repeat(161))).not.toBeNull();expect(validatePronunciationTarget('a '.repeat(13))).not.toBeNull()})})
