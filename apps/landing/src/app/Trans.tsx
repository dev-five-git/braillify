'use client'
import { Box, Flex, Image, Text, VStack } from '@devup-ui/react'
import { useEffect, useState } from 'react'

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
    <VStack flex="1" gap={['16px', null, null, '30px']}>
      <Flex
        alignItems="flex-start"
        gap={['10px', null, null, '20px']}
        justifyContent={['center', null, null, 'flex-start']}
      >
        <Box
          aria-hidden="true"
          bg="$text"
          flexShrink={0}
          h={['20px', null, null, '32px']}
          maskImage="url(/images/home/finger-point.svg)"
          maskPosition="center"
          maskRepeat="no-repeat"
          maskSize="contain"
          w={['17px', null, null, '28px']}
        />
        <Text color="$text" pos="relative" top="-2px" typography="mainTextSm">
          직접 입력해 실시간 점자 번역을 체험해보세요!
        </Text>
      </Flex>
      <Flex
        alignItems="center"
        flexDirection={['column', null, null, 'row']}
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
        <Flex aria-hidden="true">
          <Image
            alt=""
            display={['none', null, null, 'block']}
            mr="10px"
            role="presentation"
            src="/images/home/translate-arrow-circle.svg"
            w="16px"
          />
          <Image
            alt=""
            role="presentation"
            src="/images/home/translate-arrow.svg"
            transform={['rotate(0deg)', null, null, 'rotate(-90deg)']}
            w={['16px', null, null, '24px']}
          />
        </Flex>
        <TransInput
          blurPlaceholder={
            '⠴⠃⠗⠁⠊⠇⠇⠊⠋⠽⠲⠉⠵ ⠚⠒⠈⠮ ⠨⠎⠢⠱⠁⠮ ⠠⠘⠐⠪⠈⠥ ⠣⠒⠨⠻⠨⠹⠪⠐⠥ ⠰⠎⠐⠕⠚⠉⠵ ⠴⠠⠗⠥⠌⠲ ⠈⠕⠘⠒ ⠐⠣⠕⠘⠪⠐⠎⠐⠕⠕⠃⠉⠕⠊⠲\n⠴⠠⠝⠕⠙⠑⠲⠚⠎⠂ ⠠⠺⠑⠃⠠⠁⠎⠎⠑⠍⠃⠇⠽⠂ ⠠⠏⠽⠹⠕⠝⠲ ⠊⠪⠶ ⠊⠣⠜⠶⠚⠒ ⠚⠧⠒⠈⠻⠝⠠⠎ ⠇⠬⠶⠚⠂ ⠠⠍ ⠕⠌⠎⠬⠲\n\n⠨⠎⠢⠱⠁⠚⠈⠥ ⠠⠕⠲⠵ ⠑⠛⠨⠶⠕⠉ ⠊⠒⠎⠐⠮ ⠱⠈⠕⠝ ⠕⠃⠐⠱⠁⠚⠗ ⠨⠕⠁⠨⠎⠃ ⠚⠧⠁⠟⠚⠗⠘⠥⠠⠝⠬⠖'
          }
          focusPlaceholder="⠕⠈⠥⠄⠝⠀⠨⠎⠢⠱⠁⠚⠂⠀⠉⠗⠬⠶⠮⠀⠕⠃⠐⠱⠁⠚⠗⠨⠍⠠⠝⠬⠖"
          isFocused={translationFocused}
          readOnly
          value={translateToUnicode(input)}
        />
      </Flex>
    </VStack>
  )
}
