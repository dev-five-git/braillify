'use client'
import { VStack } from '@devup-ui/react'
import { useEffect, useMemo, useState } from 'react'

import { DemoArrow } from './DemoArrow'
import { DemoHeading } from './DemoHeading'
import {
  FAILED_TRACE,
  IDLE_TRACE,
  readTrace,
  RuleTrace,
  type TraceSnapshot,
} from './RuleTrace'
import { TransInput } from './TransInput'

type Translate = (input: string) => TraceSnapshot

const idleTranslate: Translate = () => IDLE_TRACE

export function Trans() {
  const [input, setInput] = useState('')
  const [translate, setTranslate] = useState<Translate>(() => idleTranslate)
  useEffect(() => {
    import('braillify').then((mod) => {
      setTranslate(() => (text: string) => {
        if (text.length === 0) return IDLE_TRACE
        try {
          return readTrace(mod.translateToUnicodeWithTrace(text))
        } catch (e) {
          console.error(e)
          return FAILED_TRACE
        }
      })
    })
  }, [])

  // 한 번의 점역으로 점자 출력과 규칙 목록을 모두 얻는다.
  const trace = useMemo(() => translate(input), [translate, input])

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
          value={trace.braille}
        />
      </VStack>
      <RuleTrace trace={trace} />
    </VStack>
  )
}
