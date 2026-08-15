'use client'

import { Box, Button, Flex, Grid, Input, Text, VStack } from '@devup-ui/react'
import { useMemo, useState } from 'react'

import type { CopyState } from '@/components/translator/TranslationOutput'
import {
  brailleMasksToString,
  type DotNumber,
  mirrorBrailleMask,
  parseBrailleString,
  toggleBrailleDot,
} from '@/lib/braille-editor'
import { copyText } from '@/lib/clipboard'

const DOT_LAYOUT = [1, 2, 3, 4, 5, 6] as const

const COPY_LABEL = {
  copied: '복사됨',
  error: '복사 실패',
  idle: '복사',
} as const satisfies Record<CopyState, string>

export function BrailleEditor() {
  const [cells, setCells] = useState<number[]>([0])
  const [intaglio, setIntaglio] = useState(false)
  const [importInput, setImportInput] = useState('')
  const [importError, setImportError] = useState<string | null>(null)
  const [copyState, setCopyState] = useState<CopyState>('idle')
  const previewMasks = useMemo(
    () => (intaglio ? cells.map(mirrorBrailleMask) : cells),
    [cells, intaglio],
  )
  const preview = useMemo(
    () => brailleMasksToString(previewMasks),
    [previewMasks],
  )

  const importBraille = () => {
    const value = importInput.trim()
    if (!value) {
      setImportError('점자 문자열을 붙여넣어 주세요.')
      return
    }

    const parsed = parseBrailleString(value)
    if (!parsed) {
      setImportError('U+2800 범위의 점자 문자만 사용할 수 있습니다.')
      return
    }

    setCells(parsed.length ? parsed : [0])
    setImportInput('')
    setImportError(null)
    setCopyState('idle')
  }

  const copyPreview = async () => {
    try {
      await copyText(preview)
      setCopyState('copied')
    } catch {
      setCopyState('error')
    }
  }

  return (
    <VStack gap="16px" pb="12px">
      <EditorCard>
        <Flex alignItems="center" gap="12px" justifyContent="space-between">
          <Text fontWeight="600" typography="sidebarBody">
            미리보기
          </Text>
          <Flex alignItems="center" gap="12px">
            <Flex alignItems="center" gap="8px">
              <Text as="label" color="$caption" typography="sidebarCaption">
                좌우 반전
              </Text>
              <Button
                aria-label="좌우 반전 미리보기"
                aria-pressed={intaglio}
                bg={intaglio ? '$primary' : '$disabledBackground'}
                border="none"
                borderRadius="999px"
                cursor="pointer"
                h="22px"
                onClick={() => setIntaglio((current) => !current)}
                position="relative"
                type="button"
                w="38px"
              >
                <Box
                  bg="$containerBackground"
                  borderRadius="50%"
                  h="16px"
                  left={intaglio ? '19px' : '3px'}
                  position="absolute"
                  top="3px"
                  transition="left 150ms ease"
                  w="16px"
                />
              </Button>
            </Flex>
            <OutlineButton onClick={copyPreview}>
              {COPY_LABEL[copyState]}
            </OutlineButton>
          </Flex>
        </Flex>
        <Box
          aria-live="polite"
          fontFamily="Segoe UI Symbol, sans-serif"
          fontSize="32px"
          letterSpacing="4px"
          lineHeight="1.4"
          minH="64px"
          py="6px"
          wordBreak="break-all"
        >
          {preview}
        </Box>
      </EditorCard>

      <EditorCard>
        <Text fontWeight="600" typography="sidebarBody">
          점자 가져오기
        </Text>
        <Input
          aria-invalid={Boolean(importError)}
          aria-label="가져올 점자 문자열"
          bg="$background"
          border="1px solid $border"
          borderRadius="8px"
          onChange={(event) => {
            setImportInput(event.target.value)
            setImportError(null)
          }}
          p="12px"
          placeholder="점자 문자열을 붙여넣으세요 (U+2800 범위)"
          value={importInput}
        />
        {importError && (
          <Text color="$error" role="alert" typography="sidebarCaption">
            {importError}
          </Text>
        )}
        <Button
          bg="$primary"
          border="none"
          borderRadius="8px"
          color="$base"
          cursor="pointer"
          onClick={importBraille}
          py="12px"
          type="button"
          typography="button"
        >
          가져오기
        </Button>
      </EditorCard>

      <EditorCard>
        <Flex alignItems="center" gap="12px" justifyContent="space-between">
          <Text fontWeight="600" typography="sidebarBody">
            점자 셀 편집 ({cells.length}셀)
          </Text>
          <Flex gap="8px">
            <OutlineButton
              onClick={() => setCells((current) => [...current, 0])}
            >
              + 셀
            </OutlineButton>
            <Button
              bg="transparent"
              border="1px solid $error"
              borderRadius="8px"
              color="$error"
              cursor="pointer"
              onClick={() => {
                setCells([0])
                setCopyState('idle')
              }}
              px="14px"
              py="7px"
              type="button"
              typography="sidebarBody"
            >
              초기화
            </Button>
          </Flex>
        </Flex>
        <Grid
          gap="16px"
          gridTemplateColumns="repeat(auto-fill, minmax(80px, 1fr))"
          py="4px"
        >
          {cells.map((mask, index) => (
            <EditableBrailleCell
              key={index}
              index={index}
              mask={mask}
              onRemove={() => {
                setCells((current) =>
                  current.length === 1
                    ? [0]
                    : current.filter((_, cellIndex) => cellIndex !== index),
                )
                setCopyState('idle')
              }}
              onToggleDot={(dot) => {
                setCells((current) =>
                  current.map((cell, cellIndex) =>
                    cellIndex === index ? toggleBrailleDot(cell, dot) : cell,
                  ),
                )
                setCopyState('idle')
              }}
            />
          ))}
        </Grid>
      </EditorCard>

      <EditorCard>
        <Text fontWeight="600" typography="sidebarBody">
          점 번호
        </Text>
        <Text color="$caption" typography="sidebarCaption">
          왼쪽: 1·2·3 / 오른쪽: 4·5·6
        </Text>
      </EditorCard>
    </VStack>
  )
}

function EditableBrailleCell({
  index,
  mask,
  onRemove,
  onToggleDot,
}: {
  index: number
  mask: number
  onRemove: () => void
  onToggleDot: (dot: DotNumber) => void
}) {
  return (
    <VStack alignItems="center" gap="6px">
      <Text color="$caption" typography="sidebarCaption">
        #{index + 1}
      </Text>
      <Box bg="$background" borderRadius="12px" px="14px" py="12px">
        <Grid
          gap="6px"
          gridAutoFlow="column"
          gridTemplateColumns="repeat(2, 22px)"
          gridTemplateRows="repeat(3, 22px)"
        >
          {DOT_LAYOUT.map((dot) => {
            const active = (mask & (1 << (dot - 1))) !== 0

            return (
              <Button
                key={dot}
                aria-label={`${index + 1}번 셀 점 ${dot}`}
                aria-pressed={active}
                bg={active ? '$primary' : '$containerBackground'}
                border="1.5px solid $border"
                borderRadius="50%"
                cursor="pointer"
                h="22px"
                onClick={() => onToggleDot(dot)}
                p="0"
                type="button"
                w="22px"
              />
            )
          })}
        </Grid>
      </Box>
      <Button
        aria-label={`${index + 1}번 셀 삭제`}
        bg="transparent"
        border="none"
        color="$caption"
        cursor="pointer"
        fontSize="18px"
        lineHeight="1"
        onClick={onRemove}
        p="0"
        type="button"
      >
        ×
      </Button>
    </VStack>
  )
}

function EditorCard({ children }: { children: React.ReactNode }) {
  return (
    <VStack
      bg="$containerBackground"
      border="1px solid $border"
      borderRadius="12px"
      gap="10px"
      px="16px"
      py="14px"
    >
      {children}
    </VStack>
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
    <Button
      bg="$containerBackground"
      border="1px solid $border"
      borderRadius="8px"
      color="$text"
      cursor="pointer"
      onClick={onClick}
      px="14px"
      py="7px"
      type="button"
      typography="sidebarBody"
    >
      {children}
    </Button>
  )
}
