export const MODE_OPTIONS = [
  { label: '일반', value: 'general' },
  { label: '수학', value: 'math' },
  { label: '역점역', value: 'reverse' },
] as const

export type TranslateMode = (typeof MODE_OPTIONS)[number]['value']

export const MODE_CONTENT = {
  general: {
    buttonLabel: '점역하기',
    guide: '일반 텍스트를 2024 개정 한국 점자 규정에 따라 변환합니다.',
    inputLabel: '점역할 일반 텍스트',
    placeholder: '점역할 한국어 문장이나 단어를 입력하세요.',
  },
  math: {
    buttonLabel: '수학 점역하기',
    guide: '수식 전체를 $...$로 감싸고 분수는 \\frac{}{}로 입력하세요.',
    inputLabel: '점역할 LaTeX 수식',
    placeholder: '$\\frac{3}{4}$',
  },
  reverse: {
    buttonLabel: '역점역하기',
    guide:
      '한국 점자 유니코드를 한글로 변환합니다. 직접 붙여넣거나 6점 입력기를 사용하세요.',
    inputLabel: '역점역할 점자',
    placeholder: '예: ⠣⠒⠉⠻',
  },
} as const satisfies Record<
  TranslateMode,
  {
    buttonLabel: string
    guide: string
    inputLabel: string
    placeholder: string
  }
>
