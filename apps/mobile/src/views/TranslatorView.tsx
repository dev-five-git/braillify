'use client'

import { masksToString } from '@braillify/shared'
import { copyText } from '@braillify/shared'
import { pushHistory } from '@braillify/shared'
import { translate, translateReverse } from '@braillify/shared'
import { Textarea } from '@devup-ui/components'
import { Box, Flex, Text } from '@devup-ui/react'
import { useState } from 'react'

import { BrailleInput } from '../components/BrailleInput'

// 3가지 모드를 하나의 타입으로 통합
type PageMode = 'general' | 'math' | 'reverse'

const MODE_OPTIONS: { value: PageMode; label: string }[] = [
  { value: 'general', label: '일반' },
  { value: 'math', label: '수학' },
  { value: 'reverse', label: '역점역' },
]

type ResultState =
  | { kind: 'idle' }
  | { kind: 'ok'; output: string }
  | { kind: 'error'; message: string }

export function TranslatorView() {
  const [pageMode, setPageMode] = useState<PageMode>('general')
  const [text, setText] = useState('')
  const [cells, setCells] = useState<number[]>([])
  const [result, setResult] = useState<ResultState>({ kind: 'idle' })
  const [copyState, setCopyState] = useState<'idle' | 'copied' | 'error'>(
    'idle',
  )

  const isReverse = pageMode === 'reverse'

  // 번역 버튼 활성화 조건
  const canTranslate = isReverse ? cells.length > 0 : !!text.trim()

  function handleModeChange(next: PageMode) {
    setPageMode(next)
    setText('')
    setCells([])
    setResult({ kind: 'idle' })
  }

  async function handleTranslate() {
    if (!isReverse) {
      const translateMode = pageMode === 'math' ? 'math' : 'general'
      const r = await translate(text, translateMode)
      if (r.ok) {
        setResult({ kind: 'ok', output: r.braille })
        pushHistory({ source: text, braille: r.braille, mode: translateMode })
      } else {
        setResult({ kind: 'error', message: r.error })
      }
    } else {
      const r = await translateReverse(masksToString(cells))
      if (r.ok) {
        setResult({ kind: 'ok', output: r.korean })
      } else {
        setResult({ kind: 'error', message: r.error })
      }
    }
    setCopyState('idle')
  }

  async function handleCopy() {
    if (result.kind !== 'ok') return
    try {
      await copyText(result.output)
      setCopyState('copied')
      setTimeout(() => setCopyState('idle'), 1500)
    } catch {
      setCopyState('error')
      setTimeout(() => setCopyState('idle'), 1500)
    }
  }

  return (
    <Box px="20px" py="24px">
      <Text as="h1" fontSize="24px" fontWeight={700} m={0} mb="6px">
        점역기
      </Text>
      <Text as="p" color="$textMuted" fontSize="14px" m={0} mb="16px">
        {isReverse
          ? '점을 눌러 점자를 조합하고 역점역합니다.'
          : pageMode === 'math'
            ? 'LaTeX 수식을 $...$로 감싸 입력하면 수학 점자로 변환합니다.'
            : '한글 텍스트를 입력하면 2024 개정 한국 점자 규정에 따라 점역합니다.'}
      </Text>

      {/* 모드 토글: 일반 | 수학 | 역점역 */}
      <Flex
        bg="$surface"
        border="1px solid"
        borderColor="$border"
        borderRadius="999px"
        display="inline-flex"
        gap="4px"
        mb="16px"
        p="4px"
      >
        {MODE_OPTIONS.map(({ value, label }) => {
          const active = pageMode === value
          return (
            <Box
              key={value}
              aria-selected={active}
              as="button"
              bg={active ? '$primary' : 'transparent'}
              border={0}
              borderRadius="999px"
              color={active ? '$primaryText' : '$textMuted'}
              fontSize="13px"
              fontWeight={active ? 600 : 500}
              onClick={() => handleModeChange(value)}
              px="18px"
              py="6px"
              role="tab"
              tabIndex={0}
              type="button"
            >
              {label}
            </Box>
          )
        })}
      </Flex>

      {/* Input card */}
      <Box
        bg="$surface"
        border="1px solid"
        borderColor="$border"
        borderRadius="12px"
        pb="12px"
        pt="14px"
        px="16px"
      >
        <Flex
          alignItems="baseline"
          borderBottom="1px solid"
          borderColor="$border"
          justifyContent="space-between"
          pb="8px"
        >
          <Text fontSize="14px" fontWeight={600}>
            {isReverse ? '점자 입력' : '입력 텍스트'}
          </Text>
          <Text color="$textSubtle" fontSize="12px">
            {isReverse ? `${cells.length}셀` : `${text.length}자`}
          </Text>
        </Flex>

        <Box py="8px">
          {isReverse ? (
            <BrailleInput cells={cells} onChange={setCells} />
          ) : (
            <Textarea
              onChange={(e) => setText(e.target.value)}
              placeholder={
                pageMode === 'math'
                  ? 'LaTeX 수식을 $...$로 감싸 입력하세요. 예: $\\frac{3}{4}$'
                  : '점역할 텍스트를 입력하세요...'
              }
              rows={4}
              value={text}
            />
          )}
        </Box>

        <Flex
          alignItems="center"
          borderColor="$border"
          borderTop="1px solid"
          justifyContent="space-between"
          pt="8px"
        >
          <Text color="$textSubtle" fontSize="12px">
            {isReverse
              ? '역점역 모드'
              : pageMode === 'math'
                ? '수학 모드 (LaTeX)'
                : '일반 모드'}
          </Text>
          <Box
            as="button"
            bg="$primary"
            border={0}
            borderRadius="8px"
            color="$primaryText"
            cursor={canTranslate ? 'pointer' : 'not-allowed'}
            disabled={!canTranslate}
            fontSize="13px"
            fontWeight={600}
            onClick={handleTranslate}
            opacity={canTranslate ? 1 : 0.4}
            px="18px"
            py="10px"
            type="button"
          >
            {isReverse ? '역점역하기' : '점역하기'}
          </Box>
        </Flex>
      </Box>

      {/* Result */}
      <Flex
        alignItems="center"
        aria-live="polite"
        justifyContent="center"
        minH="120px"
        mt="28px"
        textAlign="center"
      >
        {result.kind === 'idle' && (
          <Text color="$textMuted" fontSize="14px">
            {isReverse
              ? '점자를 입력하고 역점역을 시작해보세요'
              : '텍스트를 입력하고 점역을 시작해보세요'}
          </Text>
        )}
        {result.kind === 'ok' && (
          <Flex alignItems="center" flexDirection="column" gap="16px" w="100%">
            <Box
              color="$text"
              fontSize={isReverse ? '20px' : '28px'}
              letterSpacing={isReverse ? '0px' : '2px'}
              lineHeight={1.6}
              wordBreak="break-all"
            >
              {result.output}
            </Box>
            <Box
              as="button"
              bg="$surface"
              border="1px solid"
              borderColor="$border"
              borderRadius="8px"
              color="$text"
              fontSize="13px"
              fontWeight={500}
              onClick={handleCopy}
              px="18px"
              py="8px"
              type="button"
            >
              {copyState === 'copied'
                ? '복사됨'
                : copyState === 'error'
                  ? '복사 실패'
                  : '복사'}
            </Box>
          </Flex>
        )}
        {result.kind === 'error' && (
          <Text
            as="p"
            bg="$dangerSoft"
            border="1px solid"
            borderColor="$dangerSoftBorder"
            borderRadius="8px"
            color="$danger"
            fontSize="14px"
            m={0}
            px="16px"
            py="12px"
            w="100%"
          >
            {result.message}
          </Text>
        )}
      </Flex>
    </Box>
  )
}
