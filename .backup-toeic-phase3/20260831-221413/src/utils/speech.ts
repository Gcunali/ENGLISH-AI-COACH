const MARKDOWN_LINK = /\[([^\]]+)\]\([^)]+\)/g
const URL = /https?:\/\/\S+/gi
const CODE_BLOCK = /```[\s\S]*?```/g
const INLINE_MARKUP = /[`*_#>|~]/g

export function sanitizeForSpeech(input: string): string {
  return input
    .replace(CODE_BLOCK, ' ')
    .replace(MARKDOWN_LINK, '$1')
    .replace(URL, 'link')
    .replace(/^\s*[-+•]\s+/gm, '')
    .replace(/^\s*\d+[.)]\s+/gm, '')
    .replace(INLINE_MARKUP, '')
    .replace(/\s+/g, ' ')
    .trim()
}

export function takeCompleteSentences(input: string): { complete: string[]; remainder: string } {
  const complete: string[] = []
  let boundary = 0
  const matcher = /[^.!?]+[.!?]+(?:["'”’]+)?(?=\s|$)/g
  for (const match of input.matchAll(matcher)) {
    complete.push(match[0].trim())
    boundary = (match.index ?? 0) + match[0].length
  }
  return { complete, remainder: input.slice(boundary).trimStart() }
}
