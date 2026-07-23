import { Box, Grid, Text } from '@devup-ui/react'

import { MIDDLE_KOREAN_FONT_FAMILY } from '@/constants/font'
import { TestStatus } from '@/types'

import { LatexText } from '../LatexText'
import TestCaseCircle from '../TestCaseCircle'
import { TestCaseDisplayBoundary } from '../TestCaseDisplayBoundary'

export function TestCaseList({ results }: { results: TestStatus[6] }) {
  return (
    <Grid gap="8px" gridTemplateColumns="repeat(auto-fill, minmax(16px, 1fr))">
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
              <TestCaseCircle isSuccess={isSuccess}>
                <Box
                  maxW="min(400px, calc(100vw - 32px))"
                  minW="240px"
                  whiteSpace="pre-wrap"
                  wordBreak="break-all"
                >
                  <Text
                    color="#FFF"
                    fontFamily={MIDDLE_KOREAN_FONT_FAMILY}
                    typography="body"
                  >
                    <LatexText>{text}</LatexText>
                    {note ? ` (${note})` : null}
                    <br />
                    정답 : <Text wordBreak="break-all">{expected}</Text>
                    <br />
                    결과 : <Text wordBreak="break-all">{actual}</Text>
                    <br />
                    {isSuccess ? '✅ 테스트 성공' : '❌ 테스트 실패'}
                    {world ? (
                      <>
                        <br />
                        점자세상 : <Text wordBreak="break-all">
                          {world}
                        </Text>{' '}
                        {worldIsSuccess ? '✅' : '❌'}
                      </>
                    ) : null}
                    {jeomsarang ? (
                      <>
                        <br />
                        점사랑 : <Text wordBreak="break-all">
                          {jeomsarang}
                        </Text>{' '}
                        {jeomsarangIsSuccess ? '✅' : '❌'}
                      </>
                    ) : null}
                  </Text>
                </Box>
              </TestCaseCircle>
            </TestCaseDisplayBoundary>
          )
        },
      )}
    </Grid>
  )
}
