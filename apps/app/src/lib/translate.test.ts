import { describe, expect, mock, test } from 'bun:test'

import {
  EMPTY_INPUT_MESSAGE,
  EMPTY_REVERSE_INPUT_MESSAGE,
  INVALID_BRAILLE_MESSAGE,
  type LoadTranslationEngine,
  MATH_BRACE_MESSAGE,
  MATH_DELIMITER_MESSAGE,
  REVERSE_TRANSLATION_ERROR_MESSAGE,
  translateGeneralText,
  translateReverseText,
  translateText,
  TRANSLATION_ERROR_MESSAGE,
} from './translate'

// braillify WASM 엔진 스텁 로더. 기본값은 "안녕" ↔ "⠣⠒⠉⠻" 왕복.
function stubEngine(
  over: Partial<{
    decodeFromUnicode: (braille: string) => string
    translateToUnicode: (text: string) => string
  }> = {},
): LoadTranslationEngine {
  return async () => ({
    decodeFromUnicode: (braille) => (braille === '⠣⠒⠉⠻' ? '안녕' : ''),
    translateToUnicode: (text) => (text === '안녕' ? '⠣⠒⠉⠻' : ''),
    ...over,
  })
}

describe('translateGeneralText', () => {
  test('정상 입력을 WASM 엔진으로 점역한다', async () => {
    await expect(translateGeneralText('안녕', stubEngine())).resolves.toBe(
      '⠣⠒⠉⠻',
    )
  })

  test('공백뿐인 입력은 엔진을 호출하지 않고 거절한다', async () => {
    const translateToUnicode = mock(() => '호출되지 않아야 함')

    await expect(
      translateGeneralText(' \n\t', stubEngine({ translateToUnicode })),
    ).rejects.toThrow(EMPTY_INPUT_MESSAGE)
    expect(translateToUnicode).not.toHaveBeenCalled()
  })

  test('엔진 오류를 사용자용 메시지로 변환한다', async () => {
    const loadEngine = stubEngine({
      translateToUnicode: () => {
        throw new Error('지원하지 않는 문자: 😀')
      },
    })

    await expect(translateGeneralText('😀', loadEngine)).rejects.toThrow(
      TRANSLATION_ERROR_MESSAGE,
    )
  })

  test('엔진이 던진 빈 입력 오류는 그대로 전달한다', async () => {
    const loadEngine = stubEngine({
      translateToUnicode: () => {
        throw new Error(EMPTY_INPUT_MESSAGE)
      },
    })

    await expect(translateGeneralText('안녕', loadEngine)).rejects.toThrow(
      EMPTY_INPUT_MESSAGE,
    )
  })

  test('로더를 주입하지 않으면 실제 braillify 패키지를 불러온다', async () => {
    const translateToUnicode = mock(() => '⠣⠒⠉⠻')
    mock.module('braillify', () => ({
      decodeFromUnicode: () => '안녕',
      translateToUnicode,
    }))

    await expect(translateGeneralText('안녕')).resolves.toBe('⠣⠒⠉⠻')
    expect(translateToUnicode).toHaveBeenCalledWith('안녕')
  })
})

describe('translateText math mode', () => {
  test('올바른 $...$ LaTeX를 정규화해 엔진에 전달한다', async () => {
    const translateToUnicode = mock(() => '⠼⠙⠌⠉')

    await expect(
      translateText(
        '  $\\frac{3}{4}$  ',
        'math',
        stubEngine({ translateToUnicode }),
      ),
    ).resolves.toBe('⠼⠙⠌⠉')
    expect(translateToUnicode).toHaveBeenCalledWith('$\\frac{3}{4}$')
  })

  test('$ 구분자가 누락된 수식을 거절한다', async () => {
    await expect(
      translateText('\\frac{3}{4}', 'math', stubEngine()),
    ).rejects.toThrow(MATH_DELIMITER_MESSAGE)
  })

  test('중괄호 짝이 맞지 않는 수식을 거절한다', async () => {
    await expect(
      translateText('$\\frac{3}{4$', 'math', stubEngine()),
    ).rejects.toThrow(MATH_BRACE_MESSAGE)
  })

  test('수식 내부의 $ 구분자와 닫는 중괄호 선행을 거절한다', async () => {
    await expect(translateText('$x$y$', 'math', stubEngine())).rejects.toThrow(
      MATH_DELIMITER_MESSAGE,
    )
    await expect(translateText('$}x$', 'math', stubEngine())).rejects.toThrow(
      MATH_BRACE_MESSAGE,
    )
  })

  test('일반 모드는 수학 검증 없이 엔진으로 전달한다', async () => {
    await expect(translateText('안녕', 'general', stubEngine())).resolves.toBe(
      '⠣⠒⠉⠻',
    )
  })
})

describe('translateReverseText', () => {
  test('점자 유니코드를 한글로 역점역한다', async () => {
    await expect(translateReverseText('  ⠣⠒⠉⠻  ', stubEngine())).resolves.toBe(
      '안녕',
    )
  })

  test('로더를 주입하지 않으면 실제 braillify 패키지로 역점역한다', async () => {
    mock.module('braillify', () => ({
      decodeFromUnicode: (braille: string) =>
        braille === '⠣⠒⠉⠻' ? '안녕' : '',
      translateToUnicode: () => '',
    }))

    await expect(translateText('⠣⠒⠉⠻', 'reverse')).resolves.toBe('안녕')
  })

  test('빈 입력과 일반 문자가 섞인 입력을 거절한다', async () => {
    await expect(translateReverseText('   ', stubEngine())).rejects.toThrow(
      EMPTY_REVERSE_INPUT_MESSAGE,
    )
    await expect(translateReverseText('⠣안녕', stubEngine())).rejects.toThrow(
      INVALID_BRAILLE_MESSAGE,
    )
    await expect(translateReverseText('⡀', stubEngine())).rejects.toThrow(
      INVALID_BRAILLE_MESSAGE,
    )
  })

  test('빈 점자 셀(U+2800)만 있으면 역점역할 점자가 없다고 알린다', async () => {
    await expect(translateReverseText('⠀ ⠀', stubEngine())).rejects.toThrow(
      EMPTY_REVERSE_INPUT_MESSAGE,
    )
  })

  test('디코딩 결과가 비어 있으면 역점역 실패로 처리한다', async () => {
    const loadEngine = stubEngine({ decodeFromUnicode: () => '' })

    await expect(translateReverseText('⠣⠒⠉⠻', loadEngine)).rejects.toThrow(
      REVERSE_TRANSLATION_ERROR_MESSAGE,
    )
  })

  test('디코더 오류를 사용자용 메시지로 변환한다', async () => {
    const loadEngine = stubEngine({
      decodeFromUnicode: () => {
        throw new Error('decode failed')
      },
    })

    await expect(translateReverseText('⠣⠒⠉⠻', loadEngine)).rejects.toThrow(
      REVERSE_TRANSLATION_ERROR_MESSAGE,
    )
  })

  test('디코더가 돌려준 검증 오류는 그대로 전달한다', async () => {
    const loadEngine = stubEngine({
      decodeFromUnicode: () => {
        throw new Error(INVALID_BRAILLE_MESSAGE)
      },
    })

    await expect(translateReverseText('⠣⠒⠉⠻', loadEngine)).rejects.toThrow(
      INVALID_BRAILLE_MESSAGE,
    )
  })
})
