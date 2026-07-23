import { css, Flex, Image, VStack } from '@devup-ui/react'
import { Text } from '@devup-ui/react'

import { Table, Tbody, Td, Th, Thead, Tr } from '@/components/test-case/table'
import { TestStatus } from '@/types'

import { LatexText } from '../LatexText'
import { TestCaseDisplayBoundary } from '../TestCaseDisplayBoundary'

function CompetitorCell({
  value,
  isSuccess,
}: {
  value: string
  isSuccess: boolean
}) {
  if (!value) return <Text color="#666">-</Text>
  return (
    <Flex alignItems="center" gap="4px">
      <Text
        className={css({ color: isSuccess ? '$success' : '$error' })}
        whiteSpace="nowrap"
      >
        {isSuccess ? '일치' : '불일치'}
      </Text>
    </Flex>
  )
}

export function TestCaseTable({ results }: { results: TestStatus[6] }) {
  return (
    <Table>
      <Thead
        className={css({ display: ['none', null, null, 'table-header-group'] })}
      >
        <Tr>
          <Th>번호</Th>
          <Th>예문</Th>
          <Th>정답</Th>
          <Th>결과</Th>
          <Th>성공 여부</Th>
          <Th>점자세상</Th>
          <Th>점사랑</Th>
        </Tr>
      </Thead>
      <Tbody>
        {results.map(
          (
            [
              text,
              note,
              expected,
              actual,
              isSuccess,
              world,
              worldIsSuccess,
              jeomsarang,
              jeomsarangIsSuccess,
            ],
            index,
          ) => {
            const testCaseKey = [
              text,
              note ?? '',
              expected,
              actual,
              world,
              jeomsarang,
              index,
            ].join('::')

            return (
              <TestCaseDisplayBoundary
                key={testCaseKey}
                option="failedOnly"
                value={Number(!isSuccess)}
              >
                <Tr
                  key={`${testCaseKey}-desktop`}
                  className={css({
                    bg: isSuccess ? 'unset' : '$menuHover',
                    display: ['none', null, null, 'table-row'],
                  })}
                  data-responsive="desktop"
                >
                  <Td>{index + 1}</Td>
                  <Td>
                    <LatexText>{text}</LatexText>
                    {note ? ` (${note})` : null}
                  </Td>
                  <Td>{expected}</Td>
                  <Td>{actual}</Td>
                  <Td
                    className={css({
                      color: isSuccess ? '$success' : '$error',
                      textAlign: 'center',
                    })}
                  >
                    <Flex alignItems="center" gap="4px">
                      <Text whiteSpace="nowrap">
                        {isSuccess ? '성공' : '실패'}
                      </Text>
                      <Image
                        alt={isSuccess ? 'success' : 'error'}
                        boxSize="24px"
                        src={
                          isSuccess
                            ? '/images/test-case/success.svg'
                            : '/images/test-case/error.svg'
                        }
                      />
                    </Flex>
                  </Td>
                  <Td>
                    <CompetitorCell isSuccess={worldIsSuccess} value={world} />
                  </Td>
                  <Td>
                    <CompetitorCell
                      isSuccess={jeomsarangIsSuccess}
                      value={jeomsarang}
                    />
                  </Td>
                </Tr>
                <Tr
                  key={`${testCaseKey}-mobile`}
                  className={css({
                    bg: isSuccess ? 'unset' : '$menuHover',
                    display: ['table-row', null, null, 'none'],
                  })}
                  data-responsive="mobile"
                >
                  <Td className={css({ pb: '16px', pt: '10px' })}>
                    <VStack gap="8px">
                      <Flex
                        alignItems="center"
                        gap="4px"
                        justifyContent="space-between"
                        px="10px"
                      >
                        <Text>{index + 1}</Text>
                        <Image
                          alt={isSuccess ? 'success' : 'error'}
                          boxSize="24px"
                          src={
                            isSuccess
                              ? '/images/test-case/success.svg'
                              : '/images/test-case/error.svg'
                          }
                        />
                      </Flex>
                      <Flex alignItems="center" gap="10px" px="10px">
                        <Text typography="bodyBold" wordBreak="keep-all">
                          예문
                        </Text>
                        <Text>
                          <LatexText>{text}</LatexText>
                          {note ? ` (${note})` : null}
                        </Text>
                      </Flex>
                      <Flex alignItems="center" gap="10px" px="10px">
                        <Text typography="bodyBold" wordBreak="keep-all">
                          정답
                        </Text>
                        <Text>{expected}</Text>
                      </Flex>
                      <Flex alignItems="center" gap="10px" px="10px">
                        <Text typography="bodyBold" wordBreak="keep-all">
                          결과
                        </Text>
                        <Text>{actual}</Text>
                      </Flex>
                      {world ? (
                        <Flex alignItems="center" gap="10px" px="10px">
                          <Text typography="bodyBold" wordBreak="keep-all">
                            점자세상
                          </Text>
                          <Text
                            className={css({
                              color: worldIsSuccess ? '$success' : '$error',
                            })}
                          >
                            {worldIsSuccess ? '일치' : '불일치'}
                          </Text>
                        </Flex>
                      ) : null}
                      {jeomsarang ? (
                        <Flex alignItems="center" gap="10px" px="10px">
                          <Text typography="bodyBold" wordBreak="keep-all">
                            점사랑
                          </Text>
                          <Text
                            className={css({
                              color: jeomsarangIsSuccess
                                ? '$success'
                                : '$error',
                            })}
                          >
                            {jeomsarangIsSuccess ? '일치' : '불일치'}
                          </Text>
                        </Flex>
                      ) : null}
                    </VStack>
                  </Td>
                </Tr>
              </TestCaseDisplayBoundary>
            )
          },
        )}
      </Tbody>
    </Table>
  )
}
