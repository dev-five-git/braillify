'use client'
import { VStack } from '@devup-ui/react'
import { useEffect, useState } from 'react'

import { DemoArrow } from './DemoArrow'
import { MathTransInput } from './MathTransInput'
import { TransInput } from './TransInput'

/** 수식 입력 → 수학 점자 출력. 상태를 쥔 부분만 클라이언트로 분리한다. */
export function MathTransPanel() {
  const [latex, setLatex] = useState('')
  const [translateToUnicode, setTranslateToUnicode] = useState<
    (input: string) => string
  >(() => () => '')
  useEffect(() => {
    import('braillify').then((mod) => {
      setTranslateToUnicode(() => (input: string) => {
        try {
          return mod.translateToUnicode(input)
        } catch (e) {
          console.error(e)
          return '점역할 수 없는 수식이 포함되어 있습니다.'
        }
      })
    })
  }, [])

  const braille = latex.trim() ? translateToUnicode(`$${latex}$`) : ''

  return (
    <VStack
      flexDirection={[null, null, null, 'row']}
      gap={['12px', null, null, '30px']}
      h={['auto', null, null, '500px']}
      // 좌우 박스가 폭을 반씩 나누도록 호출부에서 분배한다. 화살표(가운데)는 제외.
      selectors={{
        '&>*:first-child, &>*:last-child': { flex: 1, minWidth: 0 },
      }}
    >
      <MathTransInput
        latex={latex}
        onLatexChange={setLatex}
        placeholder="수식을 입력하면 이곳에 수학 점자가 표시됩니다."
      />
      <DemoArrow />
      <TransInput
        blurPlaceholder="예: 사분의 삼 → ⠼⠙⠌⠼⠉"
        focusPlaceholder="예: 사분의 삼 → ⠼⠙⠌⠼⠉"
        readOnly
        value={braille}
      />
    </VStack>
  )
}
