'use client'

import { type DotNumber, masksToString, toggleDot } from '@braillify/shared'
import { Box, Flex, Text } from '@devup-ui/react'
import { useEffect, useRef, useState } from 'react'

import { EditableBrailleCell } from './EditableBrailleCell'

interface BrailleInputProps {
  cells: number[]
  onChange: (cells: number[]) => void
}

// 브라유 유니코드 범위: U+2800 ~ U+283F
const BRAILLE_START = 0x2800
const BRAILLE_END = 0x283f

function parseToCells(raw: string): number[] {
  const result: number[] = []
  for (const ch of raw) {
    const code = ch.codePointAt(0) ?? 0
    if (code >= BRAILLE_START && code <= BRAILLE_END) {
      result.push(code - BRAILLE_START)
    } else if (ch === ' ' || ch === '\n') {
      result.push(0) // 일반 공백·줄바꿈 → ⠀(U+2800) 점자 공백으로 보존
    }
    // 그 외 문자는 무시
  }
  return result
}

export function BrailleInput({ cells, onChange }: BrailleInputProps) {
  // 각 셀에 안정적인 고유 key를 부여하기 위한 카운터 + id 배열
  // (배열 인덱스를 key로 쓰면 삭제 시 React가 잘못된 컴포넌트를 재사용할 수 있음)
  const counter = useRef(0)
  const nextId = () => counter.current++
  const [cellIds, setCellIds] = useState<number[]>([])

  // 외부에서 cells가 초기화될 때(방향 전환 등) ids도 동기화
  useEffect(() => {
    if (cells.length === 0) setCellIds([])
  }, [cells.length])

  // ── 텍스트 직접 입력 ──────────────────────────────────────────
  function handleTextChange(e: React.ChangeEvent<HTMLTextAreaElement>) {
    const newCells = parseToCells(e.target.value)
    // 텍스트 붙여넣기로 전체 교체 → 모든 셀에 새 id 발급
    setCellIds(newCells.map(() => nextId()))
    onChange(newCells)
  }

  // ── 도트 키보드 ───────────────────────────────────────────────
  function handleToggleDot(cellIndex: number, dot: DotNumber) {
    // 점 토글은 셀 구조가 바뀌지 않으므로 id 변경 불필요
    onChange(cells.map((m, i) => (i === cellIndex ? toggleDot(m, dot) : m)))
  }

  function handleAddCell() {
    setCellIds((ids) => [...ids, nextId()])
    onChange([...cells, 0])
  }

  function handleAddSpace() {
    setCellIds((ids) => [...ids, nextId()])
    onChange([...cells, 0]) // mask 0 → ⠀ (점자 공백)
  }

  function handleBackspace() {
    if (cells.length > 0) {
      setCellIds((ids) => ids.slice(0, -1))
      onChange(cells.slice(0, -1))
    }
  }

  function handleRemoveCell(i: number) {
    setCellIds((ids) => ids.filter((_, idx) => idx !== i))
    onChange(cells.filter((_, idx) => idx !== i))
  }

  function handleClear() {
    setCellIds([])
    onChange([])
  }

  const hasContent = cells.length > 0
  const unicodeValue = masksToString(cells)

  return (
    <Box display="flex" flexDirection="column" gap="14px">
      {/* ── 섹션 1: 직접 입력 ─────────────────────────────── */}
      <Box>
        <Text color="$textSubtle" fontSize="12px" fontWeight={600} mb="6px">
          직접 입력 / 붙여넣기
        </Text>
        <Box
          // 포커스 시 테두리 강조
          _focus={{ borderColor: '$primary' }}
          as="textarea"
          bg="$background"
          border="1px solid"
          borderColor="$border"
          borderRadius="8px"
          color="$text"
          fontFamily="inherit"
          fontSize="12px"
          lineHeight={1.6}
          onChange={handleTextChange}
          outline="none"
          placeholder="점자 유니코드를 붙여넣으세요  예: ⠈⠎⠐⠕"
          px="12px"
          py="10px"
          resize="none"
          rows={2}
          value={unicodeValue}
          w="100%"
        />
        <Text color="$textSubtle" fontSize="11px" mt="4px">
          일반 키보드로는 점자 문자를 직접 타이핑하기 어려우므로 주로
          붙여넣기용입니다.
        </Text>
      </Box>

      {/* 구분선 */}
      <Flex alignItems="center" gap="10px">
        <Box bg="$border" flex={1} h="1px" />
        <Text color="$textSubtle" flexShrink={0} fontSize="11px">
          또는 도트 키보드로 조합
        </Text>
        <Box bg="$border" flex={1} h="1px" />
      </Flex>

      {/* ── 섹션 2: 도트 키보드 ───────────────────────────── */}
      <Box>
        <Text color="$textSubtle" fontSize="12px" fontWeight={600} mb="8px">
          도트 키보드
        </Text>

        {/* 셀 편집 그리드 */}
        {hasContent ? (
          <Box
            display="grid"
            gap="12px"
            gridTemplateColumns="repeat(auto-fill, minmax(80px, 1fr))"
            maxH="220px"
            mb="12px"
            overflowY="auto"
          >
            {cells.map((mask, i) => (
              <EditableBrailleCell
                key={cellIds[i] ?? i}
                index={i}
                mask={mask}
                onRemove={hasContent ? () => handleRemoveCell(i) : undefined}
                onToggleDot={(dot) => handleToggleDot(i, dot)}
              />
            ))}
          </Box>
        ) : (
          <Flex
            alignItems="center"
            border="1px dashed"
            borderColor="$border"
            borderRadius="8px"
            color="$textSubtle"
            fontSize="12px"
            justifyContent="center"
            mb="12px"
            py="16px"
          >
            왼쪽: 점 1·2·3 &nbsp;/&nbsp; 오른쪽: 점 4·5·6
          </Flex>
        )}

        {/* 액션 버튼 */}
        <Flex flexWrap="wrap" gap="8px">
          <Box
            aria-label="마지막 셀 지우기"
            as="button"
            bg="$surface"
            border="1px solid"
            borderColor="$border"
            borderRadius="8px"
            color="$text"
            cursor={!hasContent ? 'not-allowed' : 'pointer'}
            disabled={!hasContent}
            fontSize="14px"
            fontWeight={500}
            onClick={handleBackspace}
            opacity={!hasContent ? 0.4 : 1}
            px="14px"
            py="9px"
            type="button"
          >
            ⌫
          </Box>

          <Box
            as="button"
            bg="$surface"
            border="1px solid"
            borderColor="$border"
            borderRadius="8px"
            color="$text"
            cursor="pointer"
            fontSize="13px"
            fontWeight={500}
            onClick={handleAddSpace}
            px="14px"
            py="9px"
            type="button"
          >
            공백 ⠀
          </Box>

          <Box
            as="button"
            bg="$primary"
            border={0}
            borderRadius="8px"
            color="$primaryText"
            cursor="pointer"
            fontSize="13px"
            fontWeight={600}
            onClick={handleAddCell}
            px="16px"
            py="9px"
            type="button"
          >
            + 셀
          </Box>

          {hasContent && (
            <Box
              as="button"
              bg="transparent"
              border="1px solid"
              borderColor="$danger"
              borderRadius="8px"
              color="$danger"
              cursor="pointer"
              fontSize="13px"
              fontWeight={500}
              onClick={handleClear}
              px="14px"
              py="9px"
              type="button"
            >
              전체 삭제
            </Box>
          )}
        </Flex>
      </Box>
    </Box>
  )
}
