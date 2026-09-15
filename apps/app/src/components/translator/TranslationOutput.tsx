import { Button, Text, VStack } from '@devup-ui/react'

export type CopyState = 'idle' | 'copied' | 'error'

const COPY_STATUS = {
  copied: '결과를 클립보드에 복사했습니다.',
  error: '결과를 복사하지 못했습니다. 다시 시도해 주세요.',
  idle: '',
} as const satisfies Record<CopyState, string>

type TranslationOutputProps = {
  copyState: CopyState
  isReverse: boolean
  onCopy: () => void
  result: string
}

export function TranslationOutput({
  copyState,
  isReverse,
  onCopy,
  result,
}: TranslationOutputProps) {
  if (!result) {
    return (
      <VStack
        aria-label={isReverse ? '역점역 결과' : '점역 결과'}
        aria-live="polite"
        as="output"
        gap="16px"
        justifyContent="center"
        minH="200px"
        py="22px"
        textAlign="center"
      >
        <Text
          alignSelf="center"
          aria-hidden="true"
          color="$emptyBraille"
          fontSize={['36px', null, '42px']}
          letterSpacing="0.3em"
          typography="brailleWatermark"
        >
          ⠃⠗⠁⠊⠇⠇⠊⠋⠽
        </Text>
        <Text alignSelf="center" color="$caption" typography="body">
          {isReverse
            ? '점자를 입력하고 역점역을 시작해보세요'
            : '텍스트를 입력하고 점역을 시작해보세요'}
        </Text>
      </VStack>
    )
  }

  return (
    <VStack
      aria-label={isReverse ? '역점역 결과' : '점역 결과'}
      aria-live="polite"
      as="section"
      gap="24px"
      justifyContent="center"
      minH="220px"
      py="28px"
    >
      <Text
        alignSelf="center"
        aria-label={isReverse ? '역점역 결과 텍스트' : '점역 결과 텍스트'}
        as="output"
        color="$text"
        maxW="920px"
        textAlign="center"
        typography={isReverse ? 'reverseText' : 'braille'}
        userSelect="text"
        whiteSpace="pre-wrap"
        wordBreak="break-word"
      >
        {result}
      </Text>
      <Button
        alignSelf="center"
        aria-keyshortcuts="Control+Shift+C"
        bg="$containerBackground"
        border="1px solid $border"
        borderRadius="12px"
        color="$text"
        cursor="pointer"
        onClick={onCopy}
        px="18px"
        py="11px"
        type="button"
        typography="button"
      >
        결과 복사
      </Button>
      <Text
        alignSelf="center"
        color={copyState === 'error' ? '$error' : '$caption'}
        minH="22px"
        role={copyState === 'error' ? 'alert' : 'status'}
        typography="body"
      >
        {COPY_STATUS[copyState]}
      </Text>
    </VStack>
  )
}
