'use client'

import { Button, Flex, Text, VStack } from '@devup-ui/react'
import { useEffect, useRef, useState } from 'react'

import { testStatusPageUrl } from '@/constants'
import { useIntersectionObserver } from '@/hooks/useIntersectionObserver'
import type { TestStatus, TestStatusReportPageInfo } from '@/types'

import { TestCaseList } from './list/TestCaseList'
import { TestCaseTable } from './table/TestCaseTable'
import { useTestCase } from './TestCaseProvider'

interface TestCaseResultsProps {
  pageInfo: TestStatusReportPageInfo
  statusKey: string
  total: number
}

// Start fetching before the section reaches the viewport so scrolling rarely
// lands on a placeholder. Module-level so the observer is not rebuilt per render.
const OBSERVER_OPTIONS: IntersectionObserverInit = { rootMargin: '600px' }

const PAGE_BUTTON_STYLE = {
  _disabled: { cursor: 'not-allowed', opacity: 0.4 },
  border: 'solid 1px $primary',
  borderRadius: '8px',
  color: '$primary',
  cursor: 'pointer',
  px: '12px',
  py: '6px',
} as const

/**
 * Loads one page of a group's results on demand.
 *
 * Rows are deliberately not props: a client-component prop is serialized whole
 * into its route's RSC payload, and the corpus alone is 467k rows.
 */
export function TestCaseResults({
  pageInfo,
  statusKey,
  total,
}: TestCaseResultsProps) {
  const { options } = useTestCase()
  const containerRef = useRef<HTMLDivElement>(null)
  const isVisible = useIntersectionObserver(containerRef, OBSERVER_OPTIONS)
  const [page, setPage] = useState(1)
  const [results, setResults] = useState<TestStatus[6] | null>(null)
  const [hasError, setHasError] = useState(false)

  useEffect(() => {
    if (!isVisible) return

    const abortController = new AbortController()
    setResults(null)
    setHasError(false)
    fetch(testStatusPageUrl(statusKey, page), {
      signal: abortController.signal,
    })
      .then((response) => {
        if (!response.ok) throw new Error(`HTTP ${response.status}`)
        return response.json() as Promise<TestStatus[6]>
      })
      .then(setResults)
      .catch((error: unknown) => {
        if (error instanceof DOMException && error.name === 'AbortError') return
        setHasError(true)
      })

    return () => abortController.abort()
  }, [isVisible, page, statusKey])

  const startIndex = (page - 1) * pageInfo.pageSize

  return (
    <VStack ref={containerRef} gap="20px">
      {pageInfo.pageCount > 1 ? (
        <Flex
          alignItems="center"
          flexWrap="wrap"
          gap="8px"
          justifyContent="space-between"
        >
          <Text color="$caption" typography="body">
            {(startIndex + 1).toLocaleString()}–
            {Math.min(startIndex + pageInfo.pageSize, total).toLocaleString()} /{' '}
            {total.toLocaleString()}건
          </Text>
          <Flex alignItems="center" gap="8px">
            <Button
              {...PAGE_BUTTON_STYLE}
              disabled={page === 1}
              onClick={() => setPage(1)}
            >
              처음
            </Button>
            <Button
              {...PAGE_BUTTON_STYLE}
              disabled={page === 1}
              onClick={() => setPage((current) => Math.max(1, current - 1))}
            >
              이전
            </Button>
            <Text color="$text" typography="body">
              {page.toLocaleString()} / {pageInfo.pageCount.toLocaleString()}
            </Text>
            <Button
              {...PAGE_BUTTON_STYLE}
              disabled={page === pageInfo.pageCount}
              onClick={() =>
                setPage((current) => Math.min(pageInfo.pageCount, current + 1))
              }
            >
              다음
            </Button>
            <Button
              {...PAGE_BUTTON_STYLE}
              disabled={page === pageInfo.pageCount}
              onClick={() => setPage(pageInfo.pageCount)}
            >
              마지막
            </Button>
          </Flex>
        </Flex>
      ) : null}
      {hasError ? (
        <Text color="$error" typography="body">
          테스트 결과를 불러오지 못했습니다.
        </Text>
      ) : null}
      {!hasError && !results ? (
        <Text color="$caption" typography="body">
          테스트 결과를 불러오는 중입니다.
        </Text>
      ) : null}
      {results && options.type === 'table' ? (
        <TestCaseTable results={results} startIndex={startIndex} />
      ) : null}
      {results && options.type === 'list' ? (
        <TestCaseList results={results} />
      ) : null}
    </VStack>
  )
}
