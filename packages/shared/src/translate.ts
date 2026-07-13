export type TranslateMode = 'general' | 'math'

export type TranslateResult =
  | { ok: true; braille: string }
  | { ok: false; error: string }

export type ReverseTranslateResult =
  | { ok: true; korean: string }
  | { ok: false; error: string }

export async function translate(
  text: string,
  mode: TranslateMode,
): Promise<TranslateResult> {
  const trimmed = text.trim()
  if (!trimmed) {
    return { ok: false, error: '텍스트를 입력해주세요.' }
  }

  if (mode === 'math') {
    const validation = validateLatex(trimmed)
    if (!validation.ok) return validation
  }

  try {
    // WASM 점역 엔진은 첫 점역 시점에 동적 로드한다(hydration 차단 방지).
    const { translateToUnicode } = await import('braillify')
    const braille = translateToUnicode(text)
    return { ok: true, braille }
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    return { ok: false, error: `점역 실패: ${msg}` }
  }
}

export async function translateReverse(
  braille: string,
): Promise<ReverseTranslateResult> {
  const trimmed = braille.trim()
  if (!trimmed) {
    return { ok: false, error: '점자를 입력해주세요.' }
  }
  try {
    const { decodeFromUnicode } = await import('braillify')
    const korean = decodeFromUnicode(trimmed)
    return { ok: true, korean }
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    return { ok: false, error: `역점역 실패: ${msg}` }
  }
}

function validateLatex(text: string): TranslateResult {
  if (!text.startsWith('$') || !text.endsWith('$')) {
    return {
      ok: false,
      error: '수학 모드는 $...$ 형식으로 감싸주세요. 예: $\\frac{3}{4}$',
    }
  }
  if (text.length < 3) {
    return { ok: false, error: '$ 사이에 LaTeX 수식을 입력해주세요.' }
  }

  const dollarCount = (text.match(/\$/g) ?? []).length
  if (dollarCount % 2 !== 0) {
    return { ok: false, error: '$ 개수가 맞지 않습니다.' }
  }

  let depth = 0
  for (const ch of text) {
    if (ch === '{') depth++
    else if (ch === '}') depth--
    if (depth < 0) {
      return { ok: false, error: '중괄호 { } 짝이 맞지 않습니다.' }
    }
  }
  if (depth !== 0) {
    return { ok: false, error: '중괄호 { } 짝이 맞지 않습니다.' }
  }

  return { ok: true, braille: '' }
}
