'use client'

import { css, Flex, Text, VStack } from '@devup-ui/react'
import type { MathfieldElement } from 'mathlive'
import { useEffect, useRef, useState } from 'react'

import { normalizeFracBraces } from './normalizeFracBraces'

declare global {
  // 커스텀 엘리먼트를 JSX 에 등록하려면 React.JSX 네임스페이스 병합이 유일한 방법이다.
  // oxlint-disable-next-line no-namespace
  namespace React.JSX {
    interface IntrinsicElements {
      'math-field': React.DetailedHTMLProps<
        React.HTMLAttributes<MathfieldElement>,
        MathfieldElement
      > & { 'math-virtual-keyboard-policy'?: 'auto' | 'manual' }
    }
  }
}

export function MathTransInput({
  latex,
  onLatexChange,
  placeholder,
}: {
  latex: string
  onLatexChange: (latex: string) => void
  placeholder: string
}) {
  const [ready, setReady] = useState(false)
  const fieldRef = useRef<MathfieldElement>(null)

  useEffect(() => {
    let cancelled = false
    import('mathlive').then(({ MathfieldElement }) => {
      if (cancelled) return
      MathfieldElement.fontsDirectory = '/mathlive/fonts'
      MathfieldElement.soundsDirectory = null
      setReady(true)
    })
    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    const field = fieldRef.current
    if (!ready || !field) return
    const show = () => window.mathVirtualKeyboard.show()
    const hide = () => window.mathVirtualKeyboard.hide()
    field.addEventListener('focusin', show)
    field.addEventListener('focusout', hide)
    return () => {
      field.removeEventListener('focusin', show)
      field.removeEventListener('focusout', hide)
      window.mathVirtualKeyboard.hide()
    }
  }, [ready])

  return (
    // 출력측 TransInput 과 마찬가지로 자기 폭은 자기가 잡는다. 바깥 Flex 에
    // padding 을 두지 않는 이유는 flex-basis:0 이라도 flex 아이템의 border-box
    // 크기가 padding 합보다 작아질 수 없어서, padding 이 있으면 그만큼 출력측
    // TransInput 보다 넓어지기 때문이다.
    <Flex flex="1" minW="0">
      <VStack
        bg="$containerBackground"
        borderRadius={['16px', null, null, '30px']}
        cursor="text"
        gap="12px"
        minH="25dvh"
        onClick={() => fieldRef.current?.focus()}
        p={['16px', null, null, '40px']}
        w="100%"
      >
        <VStack flex="1" gap="8px">
          {ready && (
            <math-field
              ref={fieldRef}
              className={css({
                background: 'transparent',
                border: 'none',
                display: 'block',
                fontSize: '28px',
                width: '100%',
              })}
              math-virtual-keyboard-policy="manual"
              onInput={(e) =>
                onLatexChange(
                  normalizeFracBraces(
                    (e.target as MathfieldElement).getValue(
                      'latex-without-placeholders',
                    ),
                  ),
                )
              }
            />
          )}
          {!latex && (
            <Text
              color="$text"
              opacity={0.5}
              pointerEvents="none"
              typography="braille"
              whiteSpace="pre-line"
            >
              {placeholder}
            </Text>
          )}
        </VStack>
        <Text
          color="$text"
          minH="1.5em"
          opacity={0.7}
          typography="body"
          wordBreak="break-all"
        >
          {latex ? `LaTeX: $${latex}$` : 'LaTeX가 자동으로 생성됩니다'}
        </Text>
      </VStack>
    </Flex>
  )
}
