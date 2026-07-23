import 'katex/dist/katex.min.css'

import { readFile } from 'node:fs/promises'

import { Box, Center, css, Flex, Grid, Text, VStack } from '@devup-ui/react'
import type { Metadata } from 'next'

import { ScrollToElement } from '@/components/scroll-to-element'
import { ScrollTopButton } from '@/components/scroll-top-button'
import {
  SideBarContainer,
  SideBarProvider,
  SideBarTrigger,
} from '@/components/side-bar'
import { FailedOnlyInput } from '@/components/test-case/FailedOnlyInput'
import { TestCaseFilter } from '@/components/test-case/filter/TestCaseFilter'
import { TestCaseList } from '@/components/test-case/list/TestCaseList'
import { TestCaseTable } from '@/components/test-case/table/TestCaseTable'
import { TestCaseDisplayBoundary } from '@/components/test-case/TestCaseDisplayBoundary'
import { TestCaseFilterContainer } from '@/components/test-case/TestCaseFilterContainer'
import { TestCaseFilterValue } from '@/components/test-case/TestCaseFilterValue'
import {
  type FilterTotalMap,
  type TestCaseFilter as TestCaseFilterType,
  TestCaseProvider,
} from '@/components/test-case/TestCaseProvider'
import { TestCaseRuleContainer } from '@/components/test-case/TestCaseRuleContainer'
import { TestCaseStat } from '@/components/test-case/TestCaseStat'
import { TestCaseStatFiltered } from '@/components/test-case/TestCaseStatFiltered'
import { TestCaseTotalBoundary } from '@/components/test-case/TestCaseTotalBoundary'
import { TestCaseTypeToggle } from '@/components/test-case/TestCaseTypeToggle'
import {
  CATEGORY_PREFIX_MAP,
  createFilterMap,
  TEST_CASE_FILTERS,
  TEST_CASE_FILTERS_MAP,
} from '@/constants'
import type { TestStatusMap } from '@/types'

export const metadata: Metadata = {
  title: '테스트 케이스 - 한국·영어 점자 표준 검증',
  description:
    'Braillify의 한국어·영어 점자 변환 테스트 케이스를 표준 문서별로 확인하세요. 2024 개정 한국 점자 규정과 영어 표준 점자 규정을 기준으로 점역 결과를 검증합니다.',
  alternates: {
    canonical: '/test-case',
  },
  // Next.js does not deep-merge openGraph from the parent layout, so the
  // shared fields (siteName, locale, image) must be repeated to keep
  // social previews consistent across pages.
  openGraph: {
    type: 'website',
    locale: 'ko_KR',
    url: 'https://braillify.kr/test-case',
    siteName: 'Braillify',
    title: '테스트 케이스 | Braillify',
    description:
      '한국 점자와 영어 표준 점자 규정에 따른 Braillify 점역 테스트 케이스. 점역기 정확도를 항별로 검증합니다.',
    images: [
      {
        url: '/og-image.png',
        width: 1200,
        height: 630,
        alt: 'Braillify - 한국어 점자 변환 라이브러리',
        type: 'image/png',
      },
    ],
  },
  keywords: [
    '한국 점자 테스트',
    '점역 정확도',
    '점자 변환 테스트',
    '점역 검증',
    '2024 개정 한국 점자 규정',
    '영어 표준 점자',
    'Unified English Braille',
    'braillify 테스트',
    '점자 비교',
    '점사랑 비교',
    'World 점역 비교',
    'Braillify',
    '한국어 점자 변환',
    '한글 점역',
  ],
}

export default async function TestCasePage() {
  const [testStatus, ruleMap] = await Promise.all([
    readFile('../../test_status.json', 'utf-8').then((data) =>
      JSON.parse(data),
    ) as Promise<TestStatusMap>,
    readFile('../../rule_map.json', 'utf-8').then((data) =>
      JSON.parse(data),
    ) as Promise<Record<string, { title: string; description: string }>>,
  ])

  // Dynamically create filter map based on rule_map keys
  const filterMap = createFilterMap(Object.keys(ruleMap))

  const filterTotalMap = Object.fromEntries(
    Object.entries(filterMap).map(([key]) => [
      key,
      {
        braillify: { total: 0, fail: 0 },
        world: { total: 0, fail: 0 },
        jeomsarang: { total: 0, fail: 0 },
      },
    ]),
  ) as FilterTotalMap

  let totalTest = 0
  let totalFail = 0
  let totalWorldTest = 0
  let totalWorldFail = 0
  let totalJeomsarangTest = 0
  let totalJeomsarangFail = 0
  const cases = Object.entries(ruleMap).map(([key, value], index, self) => {
    const category = Object.entries(CATEGORY_PREFIX_MAP).find(([prefix]) =>
      key.startsWith(prefix),
    )?.[1] as TestCaseFilterType | undefined
    if (category) {
      filterTotalMap[category].braillify.total += testStatus[key][0]
      filterTotalMap[category].braillify.fail += testStatus[key][1]
      filterTotalMap[category].world.total += testStatus[key][2]
      filterTotalMap[category].world.fail += testStatus[key][3]
      filterTotalMap[category].jeomsarang.total += testStatus[key][4]
      filterTotalMap[category].jeomsarang.fail += testStatus[key][5]
    }

    totalTest += testStatus[key][0]
    totalFail += testStatus[key][1]
    totalWorldTest += testStatus[key][2]
    totalWorldFail += testStatus[key][3]
    totalJeomsarangTest += testStatus[key][4]
    totalJeomsarangFail += testStatus[key][5]

    const isBut = value.title.includes('다만')
    const currentClause = key.match(/\d+/)?.[0]
    const nextClause = self[index + 1]?.[0]?.match(/\d+/)?.[0]

    return (
      <TestCaseDisplayBoundary
        key={key}
        option="failedOnly"
        value={testStatus[key][1]}
      >
        <TestCaseDisplayBoundary option="filters" value={key}>
          <TestCaseRuleContainer key={key} exception={isBut}>
            <VStack gap="20px">
              <Flex
                alignItems="center"
                gap="20px"
                justifyContent={['space-between', null, null, 'flex-start']}
              >
                <Text
                  color="$title"
                  id={value.title}
                  scrollMarginTop="220px"
                  typography="docsTitle"
                >
                  {value.title}
                </Text>
                <TestCaseStat
                  fail={testStatus[key][1]}
                  jeomsarangFail={testStatus[key][5]}
                  jeomsarangTotal={testStatus[key][4]}
                  success={testStatus[key][0] - testStatus[key][1]}
                  total={testStatus[key][0]}
                  worldFail={testStatus[key][3]}
                  worldTotal={testStatus[key][2]}
                />
              </Flex>
              <Text color="$text" typography="body" wordBreak="keep-all">
                {value.description}
              </Text>
            </VStack>
            <TestCaseDisplayBoundary option="type" value="table">
              <TestCaseTable results={testStatus[key][6]} />
            </TestCaseDisplayBoundary>
            <TestCaseDisplayBoundary option="type" value="list">
              <TestCaseList results={testStatus[key][6]} />
            </TestCaseDisplayBoundary>
          </TestCaseRuleContainer>
          {currentClause !== nextClause && (
            <Box bg="$text" h="1px" mx={['16px', null, null, '60px']} />
          )}
        </TestCaseDisplayBoundary>
      </TestCaseDisplayBoundary>
    )
  })

  return (
    <TestCaseProvider
      filterMap={filterMap}
      filterTotalMap={filterTotalMap}
      testStatusMap={testStatus}
    >
      <SideBarProvider>
        <Box maxW="1520px" mx="auto" pb="40px" w="100%">
          <VStack
            gap="20px"
            px={['16px', null, null, '60px']}
            py={['30px', null, null, '40px']}
          >
            <VStack
              alignItems={['flex-start', null, null, 'center']}
              className={css({
                selectors: {
                  '& [aria-label="tooltip"]': {
                    zIndex: '110',
                  },
                },
              })}
              flexDir={[null, null, null, 'row']}
              gap={['10px', null, null, '20px']}
            >
              <Text as="h1" color="$title" m="0" typography="title">
                테스트 케이스
              </Text>
              <TestCaseStat
                colorPercentage={false}
                fail={totalFail}
                jeomsarangFail={totalJeomsarangFail}
                jeomsarangTotal={totalJeomsarangTest}
                showTotal
                success={totalTest - totalFail}
                total={totalTest}
                worldFail={totalWorldFail}
                worldTotal={totalWorldTest}
              />
            </VStack>
            <Text color="$text" typography="body" wordBreak="keep-all">
              모든 테스트 케이스는{' '}
              <Text
                _hover={{
                  textDecoration: 'underline',
                }}
                as="a"
                color="$link"
                href="/2024 개정 한국 점자 규정.pdf"
                target="_blank"
              >
                2024 개정 한국 점자 규정
              </Text>
              뿐 아니라{' '}
              <Text
                _hover={{
                  textDecoration: 'underline',
                }}
                as="a"
                color="$link"
                href="/Rules-of-Unified-English-Braille-2024.pdf"
                target="_blank"
              >
                영어 표준 점자 규정
              </Text>
              (
              <Text
                _hover={{
                  textDecoration: 'underline',
                }}
                as="a"
                color="$link"
                href="/Korean-UEB-3rd-edition.pdf"
                target="_blank"
              >
                국문 참고
              </Text>
              ) 등 각 분야의 공식 문서를 기반으로 작성되었습니다.
            </Text>
          </VStack>
          <TestCaseFilterContainer>
            <VStack
              alignItems={['flex-end', null, null, 'center']}
              flexDir={['column-reverse', null, null, 'row']}
              gap="12px"
              justifyContent={[null, null, null, 'space-between']}
            >
              <Flex
                gap="10px"
                overflowX="auto"
                overflowY="visible"
                pb="2px"
                px={['16px', null, null, 'unset']}
                scrollbarWidth="none"
                w="100%"
              >
                {TEST_CASE_FILTERS.map((filter) => (
                  <TestCaseFilter key={filter.value} value={filter.value}>
                    {filter.label}
                  </TestCaseFilter>
                ))}
              </Flex>
              <Flex
                alignItems="center"
                color="$primary"
                gap="10px"
                px={['16px', null, null, 'unset']}
                typography="body"
                whiteSpace="nowrap"
              >
                <Text>목록 형식</Text>
                <TestCaseTypeToggle />
                <Text>표 형식</Text>
              </Flex>
            </VStack>
            <Flex
              justifyContent="space-between"
              px={['16px', null, null, 'unset']}
            >
              <Flex alignItems="center" gap="10px">
                <FailedOnlyInput
                  className={css({
                    accentColor: '$primary',
                    cursor: 'pointer',
                    boxSize: '18px',
                  })}
                  id="failed-only"
                  name="failed-only"
                  type="checkbox"
                />
                <Text
                  as="label"
                  color="$primary"
                  cursor="pointer"
                  htmlFor="failed-only"
                  typography="body"
                >
                  실패한 케이스만 표시하기
                </Text>
              </Flex>
              <TestCaseTotalBoundary>
                <Flex alignItems="center" gap="10px">
                  <SideBarTrigger>
                    <Flex
                      _hover={{
                        opacity: 0.7,
                      }}
                      alignItems="center"
                      borderRadius="8px"
                      cursor="pointer"
                      flexDir={['row-reverse', null, null, 'row']}
                      gap="8px"
                      px="12px"
                      py="8px"
                      transition="opacity 0.2s ease"
                    >
                      <Box
                        bg="$caption"
                        boxSize="16px"
                        maskImage="url(/images/chevron.svg)"
                        maskPosition="center"
                        maskRepeat="no-repeat"
                        maskSize="contain"
                        transform={[
                          'rotate(90deg)',
                          null,
                          null,
                          'rotate(0deg)',
                        ]}
                      />
                      <Text
                        color="$primary"
                        typography="body"
                        wordBreak="keep-all"
                      >
                        목차 펼치기
                      </Text>
                    </Flex>
                  </SideBarTrigger>
                </Flex>
              </TestCaseTotalBoundary>
            </Flex>
            <Box bg="$text" h="1px" mx={['16px', null, null, 'unset']} />
          </TestCaseFilterContainer>
          <TestCaseTotalBoundary>
            <TestCaseRuleContainer
              className={css({ pb: 'unset' })}
              exception={false}
            >
              <VStack
                alignItems={[null, null, null, 'center']}
                flexDir={[null, null, null, 'row']}
                gap="20px"
                justifyContent={['space-between', null, null, 'flex-start']}
              >
                <Text color="$title" typography="title">
                  <TestCaseFilterValue map={TEST_CASE_FILTERS_MAP} />
                </Text>
                <TestCaseStatFiltered />
              </VStack>
            </TestCaseRuleContainer>
            {cases}
          </TestCaseTotalBoundary>
          <TestCaseTotalBoundary reverse>
            <Center
              flexDir="column"
              my={['30px', null, null, '40px']}
              px={['16px', null, null, '60px']}
              py={['24px', null, null, '30px']}
            >
              <Text
                color="$text"
                typography="body"
                w="100%"
                wordBreak="keep-all"
              >
                등록된 테스트가 없습니다.
              </Text>
            </Center>
          </TestCaseTotalBoundary>
          <Box bg="$text" h="1px" mx={['16px', null, null, '60px']} />
        </Box>
        <ScrollTopButton
          className={css({
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            cursor: 'pointer',
            bg: '$primary',
            borderRadius: '100px',
            flexDir: 'column',
            gap: [null, null, null, '2px'],
            pb: ['12px', null, null, '16px'],
            pt: ['12px', null, null, '12px'],
            px: ['12px', null, null, '18px'],
            position: 'fixed',
            right: '24px',
            boxSize: ['48px', null, null, '60px'],
            bottom: '24px',
            zIndex: '100',
          })}
        >
          <Box
            bg="$base"
            boxSize={['24px', null, null, '16px']}
            flexShrink="0"
            maskImage="url(/images/chevron.svg)"
            maskPosition="center"
            maskRepeat="no-repeat"
            maskSize="contain"
            rotate="90deg"
          />
          <Text
            color="$base"
            display={['none', null, null, 'initial']}
            typography="tinyBtn"
          >
            TOP
          </Text>
        </ScrollTopButton>
        <TestCaseTotalBoundary>
          {/* mobile bottom sheet */}
          <SideBarContainer
            className={css({
              maxH: '467px',
              borderTop: 'solid 1px $primary',
              borderLeft: 'solid 1px $primary',
              borderRight: 'solid 1px $primary',
              display: ['flex', null, null, 'none'],
              flexDir: 'column',
              gap: '20px',
            })}
            position="bottom"
          >
            <VStack gap="8px">
              <Flex alignItems="center" justifyContent="space-between">
                <Text
                  color="$title"
                  typography="featureTitle"
                  wordBreak="keep-all"
                >
                  <TestCaseFilterValue map={TEST_CASE_FILTERS_MAP} /> 목차
                </Text>
                <SideBarTrigger className={css({ display: 'contents' })}>
                  <Center cursor="pointer" gap="6px" p="8px">
                    <Box
                      bg="$text"
                      boxSize="24px"
                      maskImage="url(/images/close.svg)"
                      maskPosition="center"
                      maskRepeat="no-repeat"
                      maskSize="contain"
                      transform="rotate(180deg)"
                    />
                  </Center>
                </SideBarTrigger>
              </Flex>
              <Text
                color="$caption"
                typography="docsCaption"
                wordBreak="keep-all"
              >
                클릭 시 해당 항으로 이동합니다.
              </Text>
            </VStack>
            <Grid
              gap="4px"
              gridTemplateColumns="repeat(5, 1fr)"
              overflowY="auto"
            >
              {Object.entries(ruleMap).map(([key, value]) => {
                const isBut = value.title.includes('다만')
                if (isBut) return null
                return (
                  <TestCaseDisplayBoundary
                    key={key}
                    option="failedOnly"
                    value={testStatus[key][1]}
                  >
                    <TestCaseDisplayBoundary option="filters" value={key}>
                      <SideBarTrigger asChild>
                        <ScrollToElement
                          className={css({ display: 'contents' })}
                          elementId={value.title}
                        >
                          <Center
                            key={key}
                            _active={{
                              bg: '$menuActive',
                            }}
                            _hover={{
                              bg: '$menuHover',
                            }}
                            cursor="pointer"
                            flexDir="column"
                            px="12px"
                            py="3px"
                            transition="background-color 0.1s ease"
                          >
                            <Text
                              _active={{
                                color: '$primary',
                              }}
                              _hover={{
                                color: '$primary',
                              }}
                              color="$primary"
                              typography="body"
                              wordBreak="keep-all"
                            >
                              {value.title.replace(/[^\d~]/g, '')}
                            </Text>
                          </Center>
                        </ScrollToElement>
                      </SideBarTrigger>
                    </TestCaseDisplayBoundary>
                  </TestCaseDisplayBoundary>
                )
              })}
            </Grid>
          </SideBarContainer>
          {/* desktop side sheet */}
          <SideBarContainer
            className={css({
              maxH: '800px',
              top: 'calc(50% - 400px)',
              borderTop: 'solid 1px $primary',
              borderLeft: 'solid 1px $primary',
              borderBottom: 'solid 1px $primary',
              display: ['none', null, null, 'flex'],
              flexDir: 'column',
              gap: '20px',
            })}
          >
            <VStack gap="8px">
              <Flex alignItems="center" justifyContent="space-between">
                <Text
                  color="$title"
                  typography="featureTitle"
                  wordBreak="keep-all"
                >
                  <TestCaseFilterValue map={TEST_CASE_FILTERS_MAP} />
                  목차
                </Text>
                <SideBarTrigger className={css({ display: 'contents' })}>
                  <Center cursor="pointer" gap="6px" p="8px">
                    <Box
                      bg="$text"
                      boxSize="24px"
                      maskImage="url(/images/close.svg)"
                      maskPosition="center"
                      maskRepeat="no-repeat"
                      maskSize="contain"
                      transform="rotate(180deg)"
                    />
                  </Center>
                </SideBarTrigger>
              </Flex>
              <Text
                color="$caption"
                typography="docsCaption"
                wordBreak="keep-all"
              >
                클릭 시 해당 항으로 이동합니다.
              </Text>
            </VStack>
            <Grid
              gap="4px"
              gridTemplateColumns="repeat(5, 1fr)"
              overflowY="auto"
            >
              {Object.entries(ruleMap).map(([key, value]) => {
                const isBut = value.title.includes('다만')
                if (isBut) return null
                return (
                  <TestCaseDisplayBoundary
                    key={key}
                    option="failedOnly"
                    value={testStatus[key][1]}
                  >
                    <TestCaseDisplayBoundary option="filters" value={key}>
                      <ScrollToElement
                        className={css({ display: 'contents' })}
                        elementId={value.title}
                      >
                        <Center
                          key={key}
                          _active={{
                            bg: '$menuActive',
                          }}
                          _hover={{
                            bg: '$menuHover',
                          }}
                          borderRadius="1000px"
                          cursor="pointer"
                          flexDir="column"
                          px="12px"
                          py="3px"
                          transition="background-color 0.1s ease"
                        >
                          <Text
                            _active={{
                              color: '$primary',
                            }}
                            _hover={{
                              color: '$primary',
                            }}
                            color="$primary"
                            typography="body"
                            wordBreak="keep-all"
                          >
                            {value.title.replace(/[^\d~]/g, '')}
                          </Text>
                        </Center>
                      </ScrollToElement>
                    </TestCaseDisplayBoundary>
                  </TestCaseDisplayBoundary>
                )
              })}
            </Grid>
          </SideBarContainer>
        </TestCaseTotalBoundary>
      </SideBarProvider>
    </TestCaseProvider>
  )
}
