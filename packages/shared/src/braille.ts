export const BRAILLE_BASE = 0x2800
export const BRAILLE_MAX = 0x283f

export type DotNumber = 1 | 2 | 3 | 4 | 5 | 6
const DOT_BITS: Record<DotNumber, number> = {
  1: 0x01,
  2: 0x02,
  3: 0x04,
  4: 0x08,
  5: 0x10,
  6: 0x20,
}

export function dotsToMask(dots: Iterable<DotNumber>): number {
  let m = 0
  for (const d of dots) m |= DOT_BITS[d]
  return m
}

export function maskToDots(mask: number): DotNumber[] {
  const out: DotNumber[] = []
  for (const d of [1, 2, 3, 4, 5, 6] as DotNumber[]) {
    if (mask & DOT_BITS[d]) out.push(d)
  }
  return out
}

export function toggleDot(mask: number, dot: DotNumber): number {
  return mask ^ DOT_BITS[dot]
}

export function maskToBrailleChar(mask: number): string {
  return String.fromCodePoint(BRAILLE_BASE + (mask & 0x3f))
}

export function brailleCharToMask(ch: string): number | null {
  const code = ch.codePointAt(0)
  if (code == null || code < BRAILLE_BASE || code > BRAILLE_MAX) return null
  return code - BRAILLE_BASE
}

export function masksToString(masks: number[]): string {
  return masks.map(maskToBrailleChar).join('')
}

export function parseBrailleString(input: string): number[] | null {
  const masks: number[] = []
  for (const ch of input) {
    const m = brailleCharToMask(ch)
    if (m == null) return null
    masks.push(m)
  }
  return masks
}

// 음각: dot1↔dot4, dot2↔dot5, dot3↔dot6
// = swap low 3 bits with high 3 bits within a cell
export function mirrorMask(mask: number): number {
  return ((mask & 0x07) << 3) | ((mask & 0x38) >> 3)
}
