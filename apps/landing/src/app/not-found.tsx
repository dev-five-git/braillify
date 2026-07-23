import { Box, Center, Text, VStack } from '@devup-ui/react'
import type { Metadata } from 'next'

import PillButton from '@/components/home/PillButton'

export const metadata: Metadata = {
  title: '페이지를 찾을 수 없습니다 (404)',
  description:
    '요청하신 페이지를 찾을 수 없습니다. Braillify 홈으로 돌아가 한국어 점자 변환 라이브러리를 둘러보세요.',
  robots: {
    index: false,
    follow: true,
  },
}

export default function NotFound() {
  return (
    <Center
      bg="$background"
      flexDir="column"
      minH="calc(100vh - 300px)"
      px={['16px', null, null, '60px']}
      py={['60px', null, null, '120px']}
      w="100%"
    >
      <VStack
        alignItems="center"
        gap={['24px', null, null, '40px']}
        maxW="720px"
        textAlign="center"
        w="100%"
      >
        <Text
          color="$text"
          fontSize={['72px', null, null, '120px']}
          fontWeight="700"
          letterSpacing="-0.06em"
          lineHeight="1"
          typography="title"
        >
          404
        </Text>
        <VStack alignItems="center" gap={['12px', null, null, '20px']}>
          <Text as="h1" color="$title" m="0" typography="title">
            페이지를 찾을 수 없습니다
          </Text>
          <Text color="$caption" typography="bodyLg" wordBreak="keep-all">
            요청하신 페이지가 이동되었거나 더 이상 존재하지 않습니다.
            <Box as="br" />
            아래 버튼으로 Braillify 홈 또는 문서로 이동할 수 있습니다.
          </Text>
        </VStack>
        <VStack gap="16px" pt={['12px', null, null, '20px']}>
          <PillButton href="/">
            <Text color="#FFF" typography="buttonLg">
              홈으로 돌아가기
            </Text>
          </PillButton>
          <PillButton href="/docs/overview">
            <Text color="#FFF" typography="buttonLg">
              문서 둘러보기
            </Text>
          </PillButton>
        </VStack>
      </VStack>
    </Center>
  )
}
