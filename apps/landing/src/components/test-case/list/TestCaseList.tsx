import { Box, Grid, Text } from '@devup-ui/react'
import Latex from 'react-syntax-highlighter/dist/cjs/languages/hljs/latex'

import { TestStatus } from '@/types'

import TestCaseCircle from '../TestCaseCircle'
import { TestCaseDisplayBoundary } from '../TestCaseDisplayBoundary'

export function TestCaseList({ results }: { results: TestStatus[2] }) {
  return (
    <Grid gap="8px" gridTemplateColumns="repeat(auto-fill, minmax(16px, 1fr))">
      {results.map(([text, expected, actual, isSuccess], idx) => {
        const textParts = parseTextWithLaTeX(text)

        return (
          <TestCaseDisplayBoundary
            key={text + idx}
            option="failedOnly"
            value={Number(!isSuccess)}
          >
            <TestCaseCircle key={text + idx} isSuccess={isSuccess}>
              <Box minW="50vw" w="100%" whiteSpace="pre-wrap">
                <Text color="$testCaseText" typography="body">
                  {textParts.map((part, partIdx) =>
                    part.type === 'latex' ? (
                      <Latex key={partIdx}>${part.content}$</Latex>
                    ) : (
                      <span key={partIdx}>{part.content}</span>
                    ),
                  )}
                  <br />
                  정답 : <Text wordBreak="break-all">{expected}</Text>
                  <br />
                  결과 : <Text wordBreak="break-all">{actual}</Text>
                  <br />
                  {isSuccess ? '✅ 테스트 성공' : '❌ 테스트 실패'}
                </Text>
              </Box>
            </TestCaseCircle>
          </TestCaseDisplayBoundary>
        )
      })}
    </Grid>
  )
}

/**
 * This function parses text with LaTeX expressions and returns an array of parts.
 * It assumes that LaTeX is wrapped in double dollar delimiters ($$...$$).
 * Note that single dollar delimiters ($...$) are not rendered.
 * @param input - The input text to parse.
 * @returns An array of parts, where each part is either a text or a LaTeX expression.
 */
const parseTextWithLaTeX = (input: string) => {
  const parts: Array<{
    type: 'text' | 'latex'
    content: string
  }> = []
  const latexRegex = /\$\$([^$]+(?:\$(?!\$)[^$]*)*)\$\$/g
  let lastIndex = 0
  let match

  while ((match = latexRegex.exec(input)) !== null) {
    // if there is text before the LaTeX expression, add it as a text part:
    if (match.index > lastIndex) {
      const textContent = input.slice(lastIndex, match.index)
      if (textContent) {
        parts.push({ type: 'text', content: textContent })
      }
    }

    // add the LaTeX expression from double dollars:
    const latexContent = match[1]
    parts.push({ type: 'latex', content: latexContent })
    lastIndex = match.index + match[0].length
  }

  // add remaining text after the last LaTeX expression:
  if (lastIndex < input.length) {
    const remainingText = input.slice(lastIndex)
    if (remainingText) {
      parts.push({ type: 'text', content: remainingText })
    }
  }

  // if no LaTeX found, return the original text as a single text part:
  if (!parts.length) {
    parts.push({ type: 'text', content: input })
  }

  return parts
}
