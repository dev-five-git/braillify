import type { TranslateMode } from '@/constants/translation'

export const EMPTY_INPUT_MESSAGE = '점역할 내용을 입력해 주세요.'
export const MATH_DELIMITER_MESSAGE =
  'LaTeX 수식 전체를 $...$ 형식으로 입력해 주세요.'
export const MATH_BRACE_MESSAGE = 'LaTeX 중괄호의 짝을 확인해 주세요.'
export const TRANSLATION_ERROR_MESSAGE =
  '이 내용은 점역할 수 없습니다. 입력을 확인해 주세요.'
export const EMPTY_REVERSE_INPUT_MESSAGE = '역점역할 점자를 입력해 주세요.'
export const INVALID_BRAILLE_MESSAGE =
  '점자 유니코드와 공백만 입력할 수 있습니다.'
export const REVERSE_TRANSLATION_ERROR_MESSAGE =
  '이 점자는 역점역할 수 없습니다. 입력을 확인해 주세요.'

// 점역 엔진은 workspace `braillify` WASM 패키지를 웹뷰에서 직접 호출한다.
// Tauri IPC 커맨드를 두지 않고 iOS/Android/데스크톱 모두 동일한 WASM 경로를 쓴다.
type TranslationEngine = {
  decodeFromUnicode: (braille: string) => string
  translateToUnicode: (text: string) => string
}

export type LoadTranslationEngine = () => Promise<TranslationEngine>

const loadTranslationEngine: LoadTranslationEngine = () => import('braillify')

export async function translateGeneralText(
  input: string,
  loadEngine: LoadTranslationEngine = loadTranslationEngine,
): Promise<string> {
  if (input.trim().length === 0) {
    throw new Error(EMPTY_INPUT_MESSAGE)
  }

  try {
    const { translateToUnicode } = await loadEngine()
    return translateToUnicode(input)
  } catch (error) {
    if (error instanceof Error && error.message === EMPTY_INPUT_MESSAGE) {
      throw error
    }

    throw new Error(TRANSLATION_ERROR_MESSAGE, { cause: error })
  }
}

export async function translateReverseText(
  input: string,
  loadEngine: LoadTranslationEngine = loadTranslationEngine,
): Promise<string> {
  const normalizedInput = input.trim()
  if (normalizedInput.length === 0) {
    throw new Error(EMPTY_REVERSE_INPUT_MESSAGE)
  }
  if (!/^[⠀-⠿\s]+$/u.test(input)) {
    throw new Error(INVALID_BRAILLE_MESSAGE)
  }
  if (!/[⠁-⠿]/u.test(input)) {
    throw new Error(EMPTY_REVERSE_INPUT_MESSAGE)
  }

  try {
    const { decodeFromUnicode } = await loadEngine()
    const result = decodeFromUnicode(normalizedInput)
    if (!result) {
      throw new Error('empty reverse translation')
    }
    return result
  } catch (error) {
    if (
      error instanceof Error &&
      (error.message === EMPTY_REVERSE_INPUT_MESSAGE ||
        error.message === INVALID_BRAILLE_MESSAGE)
    ) {
      throw error
    }
    throw new Error(REVERSE_TRANSLATION_ERROR_MESSAGE, { cause: error })
  }
}

function validateMathInput(input: string): string {
  const normalizedInput = input.trim()

  if (
    normalizedInput.length < 3 ||
    !normalizedInput.startsWith('$') ||
    !normalizedInput.endsWith('$')
  ) {
    throw new Error(MATH_DELIMITER_MESSAGE)
  }

  let braceDepth = 0

  for (let index = 1; index < normalizedInput.length - 1; index += 1) {
    const character = normalizedInput[index]

    if (character === '\\') {
      index += 1
      continue
    }

    if (character === '$') {
      throw new Error(MATH_DELIMITER_MESSAGE)
    }

    if (character === '{') {
      braceDepth += 1
    } else if (character === '}') {
      braceDepth -= 1
      if (braceDepth < 0) {
        throw new Error(MATH_BRACE_MESSAGE)
      }
    }
  }

  if (braceDepth !== 0) {
    throw new Error(MATH_BRACE_MESSAGE)
  }

  return normalizedInput
}

export async function translateText(
  input: string,
  mode: TranslateMode,
  loadEngine: LoadTranslationEngine = loadTranslationEngine,
): Promise<string> {
  if (mode === 'reverse') {
    return translateReverseText(input, loadEngine)
  }

  if (mode === 'math') {
    return translateGeneralText(validateMathInput(input), loadEngine)
  }

  return translateGeneralText(input, loadEngine)
}
