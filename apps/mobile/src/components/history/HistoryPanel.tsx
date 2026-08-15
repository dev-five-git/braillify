'use client'

import { Box, Button, Flex, Grid, Input, Text, VStack } from '@devup-ui/react'
import { useMemo, useState } from 'react'

import { useAppState } from '@/components/shell/AppShell'
import { MODE_CONTENT } from '@/constants/translation'
import { copyText } from '@/lib/clipboard'
import type { HistoryEntry } from '@/lib/history'

type HistoryTab = 'recent' | 'favorites'

const HISTORY_TABS = ['recent', 'favorites'] as const

function formatCreatedAt(value: string): string {
  return new Intl.DateTimeFormat('ko-KR', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value))
}

export function HistoryPanel() {
  const { deleteAll, deleteEntry, entries, requestRestore, toggleFavorite } =
    useAppState()
  const [tab, setTab] = useState<HistoryTab>('recent')
  const [query, setQuery] = useState('')
  const [expandedId, setExpandedId] = useState<string | null>(null)
  const [copiedId, setCopiedId] = useState<string | null>(null)
  const [copyErrorId, setCopyErrorId] = useState<string | null>(null)
  const visibleEntries = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase()

    return entries
      .filter((entry) => (tab === 'favorites' ? entry.favorite : true))
      .filter((entry) => {
        if (!normalizedQuery) {
          return true
        }
        return (
          entry.input.toLowerCase().includes(normalizedQuery) ||
          entry.result.toLowerCase().includes(normalizedQuery)
        )
      })
  }, [entries, query, tab])

  const copyEntry = async (entry: HistoryEntry) => {
    try {
      await copyText(entry.result)
      setCopiedId(entry.id)
      setCopyErrorId(null)
      window.setTimeout(() => {
        setCopiedId((current) => (current === entry.id ? null : current))
      }, 1200)
    } catch {
      setCopiedId(null)
      setCopyErrorId(entry.id)
    }
  }

  return (
    <VStack gap="14px" pb="12px">
      <Grid
        bg="$containerBackground"
        border="1px solid $border"
        borderRadius="12px"
        gridTemplateColumns="1fr 1fr"
        p="4px"
        role="tablist"
      >
        {HISTORY_TABS.map((historyTab) => {
          const active = historyTab === tab

          return (
            <Button
              key={historyTab}
              aria-selected={active}
              bg={active ? '$primary' : 'transparent'}
              border="none"
              borderRadius="8px"
              color={active ? '$base' : '$caption'}
              cursor="pointer"
              onClick={() => setTab(historyTab)}
              py="10px"
              role="tab"
              tabIndex={0}
              type="button"
              typography="sidebarBody"
            >
              {historyTab === 'recent' ? '◉ 최근 작업' : '★ 즐겨찾기'}
            </Button>
          )
        })}
      </Grid>

      <Flex gap="8px">
        <Input
          aria-label="히스토리 검색"
          bg="$containerBackground"
          border="1px solid $border"
          borderRadius="8px"
          flex="1"
          minW="0"
          onChange={(event) => setQuery(event.target.value)}
          p="12px"
          placeholder="검색..."
          type="search"
          value={query}
        />
        <Button
          bg="$containerBackground"
          border="1px solid $border"
          borderRadius="8px"
          color="$error"
          cursor={entries.length ? 'pointer' : 'default'}
          disabled={!entries.length}
          onClick={deleteAll}
          px="12px"
          type="button"
          typography="sidebarBody"
        >
          전체 삭제
        </Button>
      </Flex>

      <VStack gap="10px">
        {visibleEntries.length === 0 && (
          <Box
            color="$caption"
            py="40px"
            textAlign="center"
            typography="sidebarBody"
          >
            {tab === 'favorites'
              ? '즐겨찾기한 항목이 없습니다.'
              : query
                ? '검색 결과가 없습니다.'
                : '아직 점역 기록이 없습니다.'}
          </Box>
        )}

        {visibleEntries.map((entry) => {
          const expanded = expandedId === entry.id

          return (
            <Box
              key={entry.id}
              as="article"
              bg="$containerBackground"
              border="1px solid $border"
              borderRadius="12px"
              px="16px"
              py="14px"
            >
              <Flex
                alignItems="center"
                gap="12px"
                justifyContent="space-between"
              >
                <VStack flex="1" gap="4px" minW="0">
                  <Text
                    fontSize="15px"
                    fontWeight="700"
                    overflow="hidden"
                    textOverflow="ellipsis"
                    whiteSpace="nowrap"
                  >
                    {entry.input || '(빈 입력)'}
                  </Text>
                  <Text
                    color="$caption"
                    fontFamily="Segoe UI Symbol, sans-serif"
                    letterSpacing="2px"
                    overflow={expanded ? 'visible' : 'hidden'}
                    textOverflow={expanded ? 'clip' : 'ellipsis'}
                    typography="sidebarBody"
                    whiteSpace={expanded ? 'normal' : 'nowrap'}
                    wordBreak="break-all"
                  >
                    {entry.result}
                  </Text>
                </VStack>

                <Flex alignItems="center" flexShrink="0" gap="6px">
                  <Button
                    aria-label={entry.favorite ? '즐겨찾기 해제' : '즐겨찾기'}
                    aria-pressed={entry.favorite}
                    bg="transparent"
                    border="none"
                    color={entry.favorite ? '$focus' : '$caption'}
                    cursor="pointer"
                    fontSize="18px"
                    lineHeight="1"
                    onClick={() => toggleFavorite(entry.id)}
                    px="2px"
                    type="button"
                  >
                    {entry.favorite ? '★' : '☆'}
                  </Button>
                  <Button
                    bg="$containerBackground"
                    border="1px solid $border"
                    borderRadius="8px"
                    color="$text"
                    cursor="pointer"
                    fontSize="12px"
                    onClick={() => copyEntry(entry)}
                    px="12px"
                    py="6px"
                    type="button"
                  >
                    {copiedId === entry.id
                      ? '복사됨'
                      : copyErrorId === entry.id
                        ? '복사 실패'
                        : '복사'}
                  </Button>
                  <Button
                    aria-label={`기록 삭제: ${entry.input}`}
                    bg="transparent"
                    border="none"
                    color="$caption"
                    cursor="pointer"
                    fontSize="16px"
                    lineHeight="1"
                    onClick={() => deleteEntry(entry.id)}
                    p="4px"
                    type="button"
                  >
                    ×
                  </Button>
                  <Button
                    aria-label={expanded ? '접기' : '펼치기'}
                    bg="transparent"
                    border="none"
                    color="$caption"
                    cursor="pointer"
                    fontSize="12px"
                    lineHeight="1"
                    onClick={() => setExpandedId(expanded ? null : entry.id)}
                    p="4px"
                    type="button"
                  >
                    {expanded ? '▲' : '▼'}
                  </Button>
                </Flex>
              </Flex>
              {expanded && (
                <VStack
                  borderTop="1px solid $border"
                  gap="10px"
                  mt="12px"
                  pt="12px"
                >
                  <Flex alignItems="center" flexWrap="wrap" gap="8px">
                    <Text color="$caption" typography="sidebarCaption">
                      {MODE_CONTENT[entry.mode].inputLabel}
                    </Text>
                    <Text color="$caption" typography="sidebarCaption">
                      {formatCreatedAt(entry.createdAt)}
                    </Text>
                  </Flex>
                  <Button
                    alignSelf="flex-end"
                    bg="$primary"
                    border="none"
                    borderRadius="8px"
                    color="$base"
                    cursor="pointer"
                    onClick={() => requestRestore(entry)}
                    px="12px"
                    py="7px"
                    type="button"
                    typography="sidebarBody"
                  >
                    점역기로 불러오기
                  </Button>
                </VStack>
              )}
            </Box>
          )
        })}
      </VStack>
    </VStack>
  )
}
