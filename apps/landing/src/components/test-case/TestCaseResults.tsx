'use client'

import { Button, Flex, Text, VStack } from '@devup-ui/react'
import { useState } from 'react'

import type { TestStatus } from '@/types'

import { TestCaseList } from './list/TestCaseList'
import { TestCaseTable } from './table/TestCaseTable'
import { useTestCase } from './TestCaseProvider'

interface TestCaseResultsProps {
  pageSize?: number
  results: TestStatus[6]
}

/**
 * Prerenders test results from the build-generated status data and paginates
 * large result sets locally without additional network requests.
 */
export function TestCaseResults({ pageSize, results }: TestCaseResultsProps) {
  const { options } = useTestCase()
  const [page, setPage] = useState(1)
  const pageCount = pageSize ? Math.ceil(results.length / pageSize) : 1
  const startIndex = pageSize ? (page - 1) * pageSize : 0
  const visibleResults = pageSize
    ? results.slice(startIndex, startIndex + pageSize)
    : results

  function handleFirstPage() {
    setPage(1)
  }

  function handlePreviousPage() {
    setPage((currentPage) => Math.max(1, currentPage - 1))
  }

  function handleNextPage() {
    setPage((currentPage) => Math.min(pageCount, currentPage + 1))
  }

  function handleLastPage() {
    setPage(pageCount)
  }

  return (
    <VStack gap="20px">
      {pageSize ? (
        <Flex
          alignItems="center"
          flexWrap="wrap"
          gap="8px"
          justifyContent="space-between"
        >
          <Text color="$caption" typography="body">
            {startIndex + 1}–{Math.min(startIndex + pageSize, results.length)} /{' '}
            {results.length.toLocaleString()}건
          </Text>
          <Flex alignItems="center" gap="8px">
            <Button
              _disabled={{ cursor: 'not-allowed', opacity: 0.4 }}
              border="solid 1px $primary"
              borderRadius="8px"
              color="$primary"
              cursor="pointer"
              disabled={page === 1}
              onClick={handleFirstPage}
              px="12px"
              py="6px"
            >
              처음
            </Button>
            <Button
              _disabled={{ cursor: 'not-allowed', opacity: 0.4 }}
              border="solid 1px $primary"
              borderRadius="8px"
              color="$primary"
              cursor="pointer"
              disabled={page === 1}
              onClick={handlePreviousPage}
              px="12px"
              py="6px"
            >
              이전
            </Button>
            <Text color="$text" typography="body">
              {page.toLocaleString()} / {pageCount.toLocaleString()}
            </Text>
            <Button
              _disabled={{ cursor: 'not-allowed', opacity: 0.4 }}
              border="solid 1px $primary"
              borderRadius="8px"
              color="$primary"
              cursor="pointer"
              disabled={page === pageCount}
              onClick={handleNextPage}
              px="12px"
              py="6px"
            >
              다음
            </Button>
            <Button
              _disabled={{ cursor: 'not-allowed', opacity: 0.4 }}
              border="solid 1px $primary"
              borderRadius="8px"
              color="$primary"
              cursor="pointer"
              disabled={page === pageCount}
              onClick={handleLastPage}
              px="12px"
              py="6px"
            >
              마지막
            </Button>
          </Flex>
        </Flex>
      ) : null}
      {options.type === 'table' ? (
        <TestCaseTable results={visibleResults} startIndex={startIndex} />
      ) : null}
      {options.type === 'list' ? (
        <TestCaseList results={visibleResults} />
      ) : null}
    </VStack>
  )
}
