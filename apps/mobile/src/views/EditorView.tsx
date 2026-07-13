'use client'

import {
  type DotNumber,
  masksToString,
  mirrorMask,
  parseBrailleString,
  toggleDot,
} from '@braillify/shared'
import { copyText } from '@braillify/shared'
import { Input, Toggle } from '@devup-ui/components'
import { Box, Flex, Text } from '@devup-ui/react'
import { useMemo, useState } from 'react'

import { EditableBrailleCell } from '../components/EditableBrailleCell'

export function EditorView() {
  const [cells, setCells] = useState<number[]>([0])
  const [intaglio, setIntaglio] = useState(false)
  const [importInput, setImportInput] = useState('')
  const [importError, setImportError] = useState<string | null>(null)
  const [copyState, setCopyState] = useState<'idle' | 'copied' | 'error'>(
    'idle',
  )

  const previewMasks = useMemo(
    () => (intaglio ? cells.map(mirrorMask) : cells),
    [cells, intaglio],
  )
  const previewString = useMemo(
    () => masksToString(previewMasks),
    [previewMasks],
  )

  function handleToggleDot(cellIndex: number, dot: DotNumber) {
    setCells((prev) =>
      prev.map((m, i) => (i === cellIndex ? toggleDot(m, dot) : m)),
    )
  }
  function handleAddCell() {
    setCells((prev) => [...prev, 0])
  }
  function handleReset() {
    setCells([0])
  }
  function handleRemoveCell(cellIndex: number) {
    setCells((prev) => {
      if (prev.length <= 1) return [0]
      return prev.filter((_, i) => i !== cellIndex)
    })
  }
  function handleImport() {
    const trimmed = importInput.trim()
    if (!trimmed) {
      setImportError('점자 문자열을 붙여넣어주세요.')
      return
    }
    const parsed = parseBrailleString(trimmed)
    if (parsed === null) {
      setImportError('U+2800 범위의 점자 문자만 사용할 수 있어요.')
      return
    }
    setImportError(null)
    setCells(parsed.length > 0 ? parsed : [0])
    setImportInput('')
  }
  async function handleCopy() {
    try {
      await copyText(previewString)
      setCopyState('copied')
      setTimeout(() => setCopyState('idle'), 1500)
    } catch {
      setCopyState('error')
      setTimeout(() => setCopyState('idle'), 1500)
    }
  }

  return (
    <Flex flexDirection="column" gap="16px" pb="12px" pt="24px" px="20px">
      <Box>
        <Text as="h1" fontSize="24px" fontWeight={700} m={0} mb="6px">
          점자 편집기
        </Text>
        <Text as="p" color="$textMuted" fontSize="14px" m={0}>
          점 단위로 직접 점자를 조합하고 음각으로 양각 인쇄 레이아웃을
          확인하세요.
        </Text>
      </Box>

      {/* Preview card */}
      <Card>
        <Flex alignItems="center" justifyContent="space-between">
          <Text fontSize="14px" fontWeight={600}>
            미리보기
          </Text>
          <Flex alignItems="center" gap="12px">
            <Flex alignItems="center" gap="8px">
              <Text as="label" color="$textMuted" fontSize="13px">
                음각
              </Text>
              <Toggle
                onChange={setIntaglio}
                value={intaglio}
                variant="switch"
              />
            </Flex>
            <OutlineButton onClick={handleCopy}>
              {copyState === 'copied'
                ? '복사됨'
                : copyState === 'error'
                  ? '복사 실패'
                  : '복사'}
            </OutlineButton>
          </Flex>
        </Flex>
        <Box
          fontSize="32px"
          letterSpacing="4px"
          lineHeight={1.4}
          minH="64px"
          py="6px"
          wordBreak="break-all"
        >
          {previewString || ' '}
        </Box>
      </Card>

      {/* Import card */}
      <Card>
        <Text fontSize="14px" fontWeight={600}>
          점자 가져오기
        </Text>
        <Input
          allowClear
          error={!!importError}
          errorMessage={importError ?? undefined}
          onChange={(e) => {
            setImportInput(e.target.value)
            setImportError(null)
          }}
          onClear={() => {
            setImportInput('')
            setImportError(null)
          }}
          placeholder="점자 문자열을 붙여넣으세요 (U+2800 범위)"
          value={importInput}
        />
        <Box
          as="button"
          bg="$primary"
          border={0}
          borderRadius="8px"
          color="$primaryText"
          fontSize="14px"
          fontWeight={600}
          onClick={handleImport}
          py="12px"
          type="button"
          w="100%"
        >
          가져오기
        </Box>
      </Card>

      {/* Cell editor card */}
      <Card>
        <Flex alignItems="center" justifyContent="space-between">
          <Text fontSize="14px" fontWeight={600}>
            점자 셀 편집 ({cells.length}셀)
          </Text>
          <Flex gap="8px">
            <OutlineButton onClick={handleAddCell}>+ 셀</OutlineButton>
            <Box
              as="button"
              bg="transparent"
              border="1px solid"
              borderColor="$danger"
              borderRadius="8px"
              color="$danger"
              fontSize="13px"
              fontWeight={600}
              onClick={handleReset}
              px="14px"
              py="7px"
              type="button"
            >
              초기화
            </Box>
          </Flex>
        </Flex>
        <Box
          display="grid"
          gap="16px"
          gridTemplateColumns="repeat(auto-fill, minmax(80px, 1fr))"
          py="4px"
        >
          {cells.map((mask, i) => (
            <EditableBrailleCell
              key={i}
              index={i}
              mask={mask}
              onRemove={() => handleRemoveCell(i)}
              onToggleDot={(dot) => handleToggleDot(i, dot)}
            />
          ))}
        </Box>
      </Card>

      {/* Dot number hint card */}
      <Card>
        <Text fontSize="14px" fontWeight={600}>
          점 번호
        </Text>
        <Text color="$textMuted" fontSize="13px">
          왼쪽: 1·2·3 / 오른쪽: 4·5·6
        </Text>
      </Card>
    </Flex>
  )
}

function Card({ children }: { children: React.ReactNode }) {
  return (
    <Flex
      bg="$surface"
      border="1px solid"
      borderColor="$border"
      borderRadius="12px"
      flexDirection="column"
      gap="10px"
      px="16px"
      py="14px"
    >
      {children}
    </Flex>
  )
}

function OutlineButton({
  children,
  onClick,
}: {
  children: React.ReactNode
  onClick: () => void
}) {
  return (
    <Box
      as="button"
      bg="$surface"
      border="1px solid"
      borderColor="$border"
      borderRadius="8px"
      color="$text"
      fontSize="13px"
      fontWeight={500}
      onClick={onClick}
      px="14px"
      py="7px"
      type="button"
    >
      {children}
    </Box>
  )
}
