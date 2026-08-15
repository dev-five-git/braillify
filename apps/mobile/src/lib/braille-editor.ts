export const BRAILLE_BLANK = '\u2800'
const BRAILLE_BASE = 0x2800
const BRAILLE_MAX = 0x283f

export type DotNumber = 1 | 2 | 3 | 4 | 5 | 6

export function createBrailleCell(dots: readonly number[]): string {
  const bitMask = dots.reduce((mask, dot) => {
    if (!Number.isInteger(dot) || dot < 1 || dot > 6) {
      throw new Error(`유효하지 않은 점 번호입니다: ${dot}`)
    }

    return mask | (1 << (dot - 1))
  }, 0)

  return String.fromCodePoint(0x2800 + bitMask)
}

export function deleteLastCharacter(value: string): string {
  return Array.from(value).slice(0, -1).join('')
}

export function toggleBrailleDot(mask: number, dot: DotNumber): number {
  return mask ^ (1 << (dot - 1))
}

export function brailleMasksToString(masks: readonly number[]): string {
  return masks
    .map((mask) => String.fromCodePoint(BRAILLE_BASE + (mask & 0x3f)))
    .join('')
}

export function parseBrailleString(value: string): number[] | null {
  const masks: number[] = []

  for (const character of value) {
    const codePoint = character.codePointAt(0)
    if (
      codePoint === undefined ||
      codePoint < BRAILLE_BASE ||
      codePoint > BRAILLE_MAX
    ) {
      return null
    }
    masks.push(codePoint - BRAILLE_BASE)
  }

  return masks
}

// 음각 인쇄 미리보기: 1↔4, 2↔5, 3↔6으로 좌우 점을 뒤집는다.
export function mirrorBrailleMask(mask: number): number {
  return ((mask & 0x07) << 3) | ((mask & 0x38) >> 3)
}
