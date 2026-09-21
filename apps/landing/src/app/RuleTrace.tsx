'use client'

import { Box, Flex, Text, VStack } from '@devup-ui/react'
import type { TraceResult } from 'braillify'

/** 한 번에 그리는 규칙 행의 최대 개수. 키 입력마다 다시 그리므로 상한을 둔다. */
const MAX_VISIBLE_RULES = 120

/** `kind` 원문 → 화면 표기. 모르는 값이면 원문을 그대로 보여준다. */
const KIND_LABEL: Record<string, string> = {
  korean: '한글',
  jamo: '자모',
  token: '기호',
  math: '수학',
  'english-ueb': '영어',
  emitter: '구조',
}

/**
 * 규칙이 하나도 잡히지 않는 경로별 설명. 점역 자체는 정상이므로
 * "적용된 규칙이 없다"고 읽히면 안 된다.
 */
const NO_RULE_NOTICE: Record<string, string> = {
  'english-ueb':
    '이 낱말은 축약 규칙 탐색이 아니라 낱말 기호표로 점역되어, 규칙 단위로 나눌 수 없습니다.',
}

/** 출력 점자의 일부를 만들어 낸 규칙 하나. WASM 객체를 평범한 값으로 옮긴 것. */
export interface TraceRule {
  section: string
  name: string
  description: string
  kind: string
  start: number
  end: number
  braille: string
}

/**
 * 한 번의 점역 결과 스냅샷.
 *
 * - `idle` — 입력이 없거나 WASM이 아직 로드되지 않음. 아무것도 그리지 않는다.
 * - `failed` — 점역이 실패함. 출력 상자가 이미 사유를 보여주므로 목록은 숨긴다.
 * - `ok` — 점역 성공. 목록을 그린다.
 */
export interface TraceSnapshot {
  status: 'idle' | 'ok' | 'failed'
  braille: string
  rules: TraceRule[]
  /** 규칙이 설명하는 출력 칸 수. */
  attributed: number
  /** 전체 출력 칸 수. `attributed`보다 크면 추적되지 않은 칸이 있다는 뜻이다. */
  total: number
  /** 입력을 처리한 엔진: `korean` | `english-ueb` | `math`. */
  path: string
}

export const IDLE_TRACE: TraceSnapshot = {
  status: 'idle',
  braille: '',
  rules: [],
  attributed: 0,
  total: 0,
  path: '',
}

export const FAILED_TRACE: TraceSnapshot = {
  status: 'failed',
  braille: '점역할 수 없는 문자가 있습니다.',
  rules: [],
  attributed: 0,
  total: 0,
  path: '',
}

/**
 * WASM `TraceResult`를 평범한 JS 값으로 복사하고 WASM 쪽 핸들을 해제한다.
 * getter 하나하나가 WASM 메모리를 읽으므로 렌더 중에 다시 만지지 않도록 한 번에 옮긴다.
 */
export function readTrace(result: TraceResult): TraceSnapshot {
  const spans = result.rules
  const snapshot: TraceSnapshot = {
    status: 'ok',
    braille: result.braille,
    attributed: result.attributed,
    total: result.total,
    path: result.path,
    rules: spans.map((span) => ({
      section: span.section,
      name: span.name,
      description: span.description,
      kind: span.kind,
      start: span.start,
      end: span.end,
      braille: span.braille,
    })),
  }
  for (const span of spans) span.free()
  result.free()
  return snapshot
}

/**
 * 항 번호 표기. `-`는 규정 항이 없는 구조 출력이고 `?`는 항 번호를 아직
 * 선언하지 않은 규칙이다. 둘 다 `제N항`으로 꾸며내지 않는다.
 *
 * 영어는 한국 점자 규정이 아니라 UEB 규정을 따르므로 `제N항`이 아닌 `§N` 표기를
 * 쓴다. 수학은 같은 규정 안의 별도 장이라 한글 제N항과 번호가 겹치므로 `수학`을
 * 붙여 구분한다. 번호 체계가 다른 규정을 같은 꼴로 적으면 출처를 잘못 읽게 된다.
 */
function sectionLabel(section: string, kind: string): string | null {
  if (section === '-') return null
  if (section === '?') return '규정 미표기'
  if (kind === 'english-ueb') return `§${section}`
  return kind === 'math' ? `수학 제${section}항` : `제${section}항`
}

/** 반열린 구간 `[start, end)`를 1부터 세는 사람 기준 표기로 옮긴다. */
function rangeLabel(start: number, end: number): string {
  if (end - start <= 1) return `${start + 1}번째 칸`
  return `${start + 1}–${end}번째 칸`
}

/** 빈 칸(U+2800)도 눈에 보이도록 칸 하나를 칩으로 그린다. */
function BrailleCells({ braille }: { braille: string }) {
  return (
    <Flex flexWrap="wrap" gap="3px">
      {Array.from(braille).map((cell, index) => (
        <Text
          key={`${cell}-${index}`}
          bg="$background"
          borderRadius="6px"
          color="$text"
          minW={['20px', null, null, '26px']}
          px="4px"
          textAlign="center"
          typography="braille"
        >
          {cell}
        </Text>
      ))}
    </Flex>
  )
}

function RuleRow({ rule }: { rule: TraceRule }) {
  const section = sectionLabel(rule.section, rule.kind)

  return (
    <Flex
      alignItems={[null, null, null, 'center']}
      as="li"
      borderTop="1px solid $border"
      flexDirection={['column', null, null, 'row']}
      gap={['8px', null, null, '16px']}
      py={['12px', null, null, '14px']}
    >
      <Box flex="none" w={[null, null, null, '104px']}>
        {section ? (
          <Text
            border="1px solid $border"
            borderRadius="1000px"
            color="$caption"
            display="inline-block"
            px="10px"
            py="4px"
            typography="tinyBtn"
            whiteSpace="nowrap"
          >
            {section}
          </Text>
        ) : null}
      </Box>
      <VStack flex="1" gap="2px" minW="0">
        <Text color="$text" typography="bodyBold" wordBreak="break-all">
          {rule.name}
        </Text>
        <Text color="$caption" typography="footer" wordBreak="break-word">
          {rule.description} · {KIND_LABEL[rule.kind] ?? rule.kind}
        </Text>
      </VStack>
      <Box flex="none">
        <BrailleCells braille={rule.braille} />
      </Box>
      <Text
        color="$caption"
        flex="none"
        textAlign={[null, null, null, 'right']}
        typography="footer"
        w={[null, null, null, '120px']}
        whiteSpace="nowrap"
      >
        {rangeLabel(rule.start, rule.end)}
      </Text>
    </Flex>
  )
}

/** 점역 결과를 만들어 낸 규칙 목록. 추적은 아직 부분적이라 덮인 범위를 함께 밝힌다. */
export function RuleTrace({ trace }: { trace: TraceSnapshot }) {
  if (trace.status !== 'ok') return null

  const isPartial = trace.attributed < trace.total
  const visibleRules = trace.rules.slice(0, MAX_VISIBLE_RULES)
  const hiddenCount = trace.rules.length - visibleRules.length
  const emptyNotice =
    trace.rules.length > 0
      ? null
      : (NO_RULE_NOTICE[trace.path] ??
        '이 입력에 대해 기록된 규칙이 아직 없습니다.')

  return (
    <VStack
      bg="$containerBackground"
      borderRadius={['16px', null, null, '30px']}
      gap={['12px', null, null, '20px']}
      p={['16px', null, null, '40px']}
      w="100%"
    >
      <Flex
        alignItems={[null, null, null, 'baseline']}
        flexDirection={['column', null, null, 'row']}
        gap={['4px', null, null, '12px']}
        justifyContent="space-between"
      >
        <Text color="$text" typography="featureTitle">
          적용 규칙
        </Text>
        {trace.total > 0 ? (
          <Text color="$caption" typography="footer" whiteSpace="nowrap">
            {trace.attributed}/{trace.total}칸 추적됨
          </Text>
        ) : null}
      </Flex>

      {isPartial ? (
        <Text color="$caption" typography="body" wordBreak="keep-all">
          출력 {trace.total}칸 가운데 {trace.attributed}칸만 규칙으로
          설명됩니다. 나머지 {trace.total - trace.attributed}칸은 아직 규칙
          추적이 붙지 않은 부분입니다.
        </Text>
      ) : null}
      {emptyNotice ? (
        <Text color="$caption" typography="body" wordBreak="keep-all">
          {emptyNotice}
        </Text>
      ) : (
        <Flex
          as="ul"
          flexDirection="column"
          listStyle="none"
          m="0"
          maxH={['320px', null, null, '420px']}
          overflowY="auto"
          p="0"
        >
          {visibleRules.map((rule, index) => (
            <RuleRow
              key={`${rule.name}-${rule.start}-${rule.end}-${index}`}
              rule={rule}
            />
          ))}
        </Flex>
      )}
      {hiddenCount > 0 ? (
        <Text color="$caption" typography="footer" wordBreak="keep-all">
          규칙 {hiddenCount}개는 목록에 표시하지 않았습니다.
        </Text>
      ) : null}
    </VStack>
  )
}
