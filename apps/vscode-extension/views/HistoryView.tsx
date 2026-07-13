'use client'

import { copyText } from '@braillify/shared'
import {
  type HistoryItem,
  listHistory,
  removeHistory,
  subscribeHistory,
  toggleFavorite,
} from '@braillify/shared'
import { Input } from '@devup-ui/components'
import { Box, Flex, Text } from '@devup-ui/react'
import { useEffect, useMemo, useState } from 'react'

type Tab = 'recent' | 'favorites'

export function HistoryView() {
  const [items, setItems] = useState<HistoryItem[]>(() => listHistory())
  const [tab, setTab] = useState<Tab>('recent')
  const [query, setQuery] = useState('')
  const [expanded, setExpanded] = useState<string | null>(null)
  const [copiedId, setCopiedId] = useState<string | null>(null)

  useEffect(() => subscribeHistory(() => setItems(listHistory())), [])

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase()
    return items
      .filter((it) => (tab === 'favorites' ? it.favorite : true))
      .filter((it) => {
        if (!q) return true
        return (
          it.source.toLowerCase().includes(q) ||
          it.braille.toLowerCase().includes(q)
        )
      })
  }, [items, tab, query])

  async function handleCopy(item: HistoryItem) {
    try {
      await copyText(item.braille)
      setCopiedId(item.id)
      setTimeout(
        () => setCopiedId((cur) => (cur === item.id ? null : cur)),
        1200,
      )
    } catch {
      // ignore
    }
  }

  return (
    <Flex flexDirection="column" gap="14px" pb="12px" pt="24px" px="20px">
      <Box>
        <Text as="h1" fontSize="24px" fontWeight={700} m={0} mb="6px">
          점역 히스토리
        </Text>
        <Text as="p" color="$textMuted" fontSize="14px" m={0}>
          최근 점역 작업 내역과 즐겨찾기를 관리합니다.
        </Text>
      </Box>

      {/* Tabs */}
      <Box
        bg="$surface"
        border="1px solid"
        borderColor="$border"
        borderRadius="12px"
        display="grid"
        gridTemplateColumns="1fr 1fr"
        p="4px"
      >
        {(['recent', 'favorites'] as const).map((t) => {
          const active = tab === t
          return (
            <Box
              key={t}
              aria-selected={active}
              as="button"
              bg={active ? '$primary' : 'transparent'}
              border={0}
              borderRadius="8px"
              color={active ? '$primaryText' : '$textMuted'}
              fontSize="13px"
              fontWeight={600}
              onClick={() => setTab(t)}
              py="10px"
              role="tab"
              tabIndex={0}
              type="button"
            >
              {t === 'recent' ? '🕐 최근 작업' : '⭐ 즐겨찾기'}
            </Box>
          )
        })}
      </Box>

      {/* Search */}
      <Input
        allowClear
        onChange={(e) => setQuery(e.target.value)}
        onClear={() => setQuery('')}
        placeholder="검색..."
        type="search"
        value={query}
      />

      {/* List */}
      <Flex flexDirection="column" gap="10px">
        {filtered.length === 0 && (
          <Box color="$textSubtle" fontSize="13px" py="40px" textAlign="center">
            {tab === 'favorites'
              ? '즐겨찾기한 항목이 없어요.'
              : query
                ? '검색 결과가 없어요.'
                : '아직 점역 기록이 없어요.'}
          </Box>
        )}
        {filtered.map((it) => {
          const isExpanded = expanded === it.id
          return (
            <Box
              key={it.id}
              bg="$surface"
              border="1px solid"
              borderColor="$border"
              borderRadius="12px"
              px="16px"
              py="14px"
            >
              <Flex
                alignItems="center"
                gap="12px"
                justifyContent="space-between"
              >
                <Flex flex={1} flexDirection="column" gap="4px" minW={0}>
                  <Box
                    fontSize="15px"
                    fontWeight={700}
                    overflow="hidden"
                    textOverflow="ellipsis"
                    whiteSpace="nowrap"
                  >
                    {it.source || '(빈 입력)'}
                  </Box>
                  <Box
                    color="$textMuted"
                    fontSize="14px"
                    letterSpacing="2px"
                    overflow={isExpanded ? 'visible' : 'hidden'}
                    textOverflow={isExpanded ? 'clip' : 'ellipsis'}
                    whiteSpace={isExpanded ? 'normal' : 'nowrap'}
                    wordBreak="break-all"
                  >
                    {it.braille}
                  </Box>
                </Flex>
                <Flex alignItems="center" gap="6px">
                  <Box
                    aria-label={it.favorite ? '즐겨찾기 해제' : '즐겨찾기'}
                    aria-pressed={it.favorite}
                    as="button"
                    bg="transparent"
                    border={0}
                    fontSize="18px"
                    lineHeight={1}
                    onClick={() => toggleFavorite(it.id)}
                    px="2px"
                    type="button"
                  >
                    {it.favorite ? '⭐' : '☆'}
                  </Box>
                  <Box
                    as="button"
                    bg="$surface"
                    border="1px solid"
                    borderColor="$border"
                    borderRadius="8px"
                    color="$text"
                    fontSize="12px"
                    fontWeight={500}
                    onClick={() => handleCopy(it)}
                    px="12px"
                    py="6px"
                    type="button"
                  >
                    {copiedId === it.id ? '복사됨' : '복사'}
                  </Box>
                  <Box
                    aria-label="삭제"
                    as="button"
                    bg="transparent"
                    border={0}
                    color="$textSubtle"
                    fontSize="14px"
                    lineHeight={1}
                    onClick={() => removeHistory(it.id)}
                    p="4px"
                    type="button"
                  >
                    ×
                  </Box>
                  <Box
                    aria-label={isExpanded ? '접기' : '펼치기'}
                    as="button"
                    bg="transparent"
                    border={0}
                    color="$textSubtle"
                    fontSize="14px"
                    lineHeight={1}
                    onClick={() => setExpanded(isExpanded ? null : it.id)}
                    p="4px"
                    type="button"
                  >
                    {isExpanded ? '▲' : '▼'}
                  </Box>
                </Flex>
              </Flex>
            </Box>
          )
        })}
      </Flex>
    </Flex>
  )
}
