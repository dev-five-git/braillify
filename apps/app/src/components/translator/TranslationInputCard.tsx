import { Button, Flex, Input, Text, VStack } from '@devup-ui/react'
import type { ReactNode } from 'react'

type TranslationInputCardProps = {
  buttonLabel: string
  // 역점역 6점 입력기. 서버에서 조합된 정적 셸이 내려온다(활성 시에만 표시).
  composerSlot: ReactNode
  errorMessage: string | null
  helpText: string
  input: string
  inputLabel: string
  isTranslating: boolean
  isReverse: boolean
  onChange: (value: string) => void
  onSubmit: () => void
  placeholder: string
}

export function TranslationInputCard({
  buttonLabel,
  composerSlot,
  errorMessage,
  helpText,
  input,
  inputLabel,
  isTranslating,
  isReverse,
  onChange,
  onSubmit,
  placeholder,
}: TranslationInputCardProps) {
  const characterCount = Array.from(input).length

  return (
    <VStack
      as="section"
      bg="$containerBackground"
      border="1px solid $border"
      borderRadius="20px"
      boxShadow="0 1px 3px rgba(34, 34, 34, 0.04)"
      overflow="hidden"
    >
      <Flex
        alignItems="center"
        borderBottom="1px solid $border"
        justifyContent="space-between"
        minH="66px"
        px="24px"
      >
        <Text as="h2" typography="inputTitle">
          {inputLabel}
        </Text>
        <Text aria-live="polite" color="$caption" typography="body">
          {characterCount}
          {isReverse ? '칸' : '자'}
        </Text>
      </Flex>

      <Input
        aria-describedby="translation-help"
        aria-invalid={errorMessage ? true : undefined}
        aria-label={inputLabel}
        as="textarea"
        bg="transparent"
        border="none"
        color="$text"
        minH={isReverse ? '158px' : '224px'}
        onChange={(event) => onChange(event.target.value)}
        onKeyDown={(event) => {
          if (event.ctrlKey && event.key === 'Enter') {
            event.preventDefault()
            onSubmit()
          }
        }}
        p="26px"
        placeholder={placeholder}
        resize="none"
        typography={isReverse ? 'brailleInput' : undefined}
        value={input}
      />

      {isReverse && composerSlot}

      <Flex
        alignItems="center"
        borderTop="1px solid $border"
        gap="18px"
        justifyContent="flex-end"
        minH="88px"
        px="24px"
      >
        <Text
          color={errorMessage ? '$error' : '$caption'}
          id="translation-help"
          role={errorMessage ? 'alert' : undefined}
          typography="body"
        >
          {errorMessage || helpText}
          {!errorMessage && (
            <Text as="span" display={['none', null, 'inline']}>
              {helpText ? ' · ' : ''}Ctrl + Enter로 변환
            </Text>
          )}
        </Text>
        <Button
          bg={
            isTranslating || input.trim().length === 0
              ? '$disabledBackground'
              : '$primary'
          }
          border="none"
          borderRadius="13px"
          color={
            isTranslating || input.trim().length === 0
              ? '$disabledText'
              : '$base'
          }
          cursor={
            isTranslating
              ? 'wait'
              : input.trim().length === 0
                ? 'default'
                : 'pointer'
          }
          disabled={isTranslating || input.trim().length === 0}
          minW="132px"
          onClick={onSubmit}
          px="22px"
          py="15px"
          type="button"
          typography="button"
        >
          {isTranslating ? '변환 중…' : buttonLabel}
        </Button>
      </Flex>
    </VStack>
  )
}
