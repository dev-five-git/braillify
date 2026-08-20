'use client'
import { VStack } from '@devup-ui/react'
import { useEffect, useState } from 'react'

import { DemoArrow } from './DemoArrow'
import { DemoHeading } from './DemoHeading'
import { TransInput } from './TransInput'

export function Trans() {
  const [input, setInput] = useState('')
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
          return '점역할 수 없는 문자가 있습니다.'
        }
      })
    })
  }, [input])

  const [inputFocused, setInputFocused] = useState(false)
  const [translationFocused, setTranslationFocused] = useState(false)

  return (
    <VStack gap={['16px', null, null, '30px']}>
      <DemoHeading>직접 입력해 실시간 점자 번역을 체험해보세요!</DemoHeading>
      <VStack
        flexDirection={[null, null, null, 'row']}
        gap={['12px', null, null, '30px']}
        h={[inputFocused ? '50dvh' : 'auto', null, null, '500px']}
      >
        <TransInput
          blurPlaceholder={
            'braillify는 한글 점역을 빠르고 안정적으로 처리하는 Rust 기반 라이브러리입니다.\nNode.js, WebAssembly, Python 등 다양한 환경에서 사용할 수 있어요.\n\n점역하고 싶은 문장이나 단어를 여기에 입력해 직접 확인해보세요!'
          }
          focusPlaceholder="이곳에 점역할 내용을 입력해주세요!"
          isFocused={inputFocused}
          onBlur={() => {
            setInputFocused(false)
            setTranslationFocused(false)
          }}
          onChange={(e) => setInput(e.target.value)}
          onFocus={() => {
            setInputFocused(true)
            setTranslationFocused(true)
          }}
          value={input}
        />
        <DemoArrow />
        <TransInput
          blurPlaceholder={
            '⠴⠃⠗⠁⠊⠇⠇⠊⠋⠽⠲⠉⠵ ⠚⠒⠈⠮ ⠨⠎⠢⠱⠁⠮ ⠠⠘⠐⠪⠈⠥ ⠣⠒⠨⠻⠨⠹⠪⠐⠥ ⠰⠎⠐⠕⠚⠉⠵ ⠴⠠⠗⠥⠌⠲ ⠈⠕⠘⠒ ⠐⠣⠕⠘⠪⠐⠎⠐⠕⠕⠃⠉⠕⠊⠲\n⠴⠠⠝⠕⠙⠑⠲⠚⠎⠂ ⠠⠺⠑⠃⠠⠁⠎⠎⠑⠍⠃⠇⠽⠂ ⠠⠏⠽⠹⠕⠝⠲ ⠊⠪⠶ ⠊⠣⠜⠶⠚⠒ ⠚⠧⠒⠈⠻⠝⠠⠎ ⠇⠬⠶⠚⠂ ⠠⠍ ⠕⠌⠎⠬⠲\n\n⠨⠎⠢⠱⠁⠚⠈⠥ ⠠⠕⠲⠵ ⠑⠛⠨⠶⠕⠉ ⠊⠒⠎⠐⠮ ⠱⠈⠕⠝ ⠕⠃⠐⠱⠁⠚⠗ ⠨⠕⠁⠨⠎⠃ ⠚⠧⠁⠟⠚⠗⠘⠥⠠⠝⠬⠖'
          }
          focusPlaceholder="⠕⠈⠥⠄⠝⠀⠨⠎⠢⠱⠁⠚⠂⠀⠉⠗⠬⠶⠮⠀⠕⠃⠐⠱⠁⠚⠗⠨⠍⠠⠝⠬⠖"
          isFocused={translationFocused}
          readOnly
          value={translateToUnicode(input)}
        />
      </VStack>
    </VStack>
  )
}
