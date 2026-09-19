import katex from 'katex'
import { Fragment } from 'react'

/**
 * Render a LaTeX expression to HTML with all KaTeX noise suppressed.
 *
 * - `strict: 'ignore'` silences "LaTeX-incompatible input" warnings
 *   (Korean text in math mode, Unicode operators like ∆/⦾/□/≯, etc.).
 * - KaTeX emits "No character metrics for '...'" via an unconditional
 *   `console.warn` regardless of `strict` (see TODO in katex.js:4966).
 *   We locally wrap `console.warn` for the duration of the render to
 *   drop only that message and restore the original immediately after.
 */
function renderKatex(latex: string, displayMode: boolean): string {
  const origWarn = console.warn
  console.warn = (...args: unknown[]) => {
    const first = args[0]
    if (typeof first === 'string' && first.startsWith('No character metrics'))
      return
    origWarn(...args)
  }
  try {
    return katex.renderToString(latex, {
      displayMode,
      strict: 'ignore',
      throwOnError: false,
    })
  } catch {
    // Defensive: katex.renderToString with throwOnError:false should not
    // throw, but if it ever does, fall back to the raw source.
    return latex
  } finally {
    console.warn = origWarn
  }
}

type Segment =
  | { kind: 'text'; content: string }
  | { kind: 'inline' | 'block'; content: string }

/**
 * Split text into plain-text and LaTeX segments using $..$ and $$..$$ pairs.
 * Honors `\$` as a literal dollar sign. An unmatched `$` is left as literal text.
 */
function splitLatex(input: string): Segment[] {
  const segments: Segment[] = []
  let buf = ''
  let i = 0
  while (i < input.length) {
    const ch = input[i]
    if (ch === '\\' && input[i + 1] === '$') {
      buf += '$'
      i += 2
      continue
    }
    if (ch === '$') {
      const isBlock = input[i + 1] === '$'
      const delim = isBlock ? '$$' : '$'
      const start = i + delim.length
      const end = input.indexOf(delim, start)
      if (end === -1) {
        buf += ch
        i += 1
        continue
      }
      if (buf) {
        segments.push({ kind: 'text', content: buf })
        buf = ''
      }
      segments.push({
        kind: isBlock ? 'block' : 'inline',
        content: input.slice(start, end),
      })
      i = end + delim.length
      continue
    }
    buf += ch
    i += 1
  }
  if (buf) segments.push({ kind: 'text', content: buf })
  return segments
}

/**
 * Renders text that may contain LaTeX expressions wrapped in $...$ or $$...$$.
 * Falls back to plain text when no `$` is present (avoids needless work).
 */
export function LatexText({ children }: { children: string }) {
  if (!children.includes('$')) return <>{children}</>
  const segments = splitLatex(children)
  return (
    <>
      {segments.map((seg, idx) => {
        if (seg.kind === 'text')
          return <Fragment key={idx}>{seg.content}</Fragment>
        const html = renderKatex(seg.content, seg.kind === 'block')
        return <span key={idx} dangerouslySetInnerHTML={{ __html: html }} />
      })}
    </>
  )
}
