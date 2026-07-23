import { Box, Flex, Text, VStack } from '@devup-ui/react'
import { Metadata } from 'next'

import TeamMemberCard from '@/components/team/TeamMemberCard'

export const metadata: Metadata = {
  title: '팀 소개',
  description:
    'Braillify를 만들어가는 팀원들을 소개합니다. 데브파이브(Devfive)와 오픈소스 기여자들이 함께 한국 점자 변환 라이브러리를 개발합니다.',
  alternates: {
    canonical: '/team',
  },
  // Next.js does not deep-merge openGraph from the parent layout, so the
  // shared fields (siteName, locale, image) must be repeated to keep
  // social previews consistent across pages.
  openGraph: {
    type: 'website',
    locale: 'ko_KR',
    url: 'https://braillify.kr/team',
    siteName: 'Braillify',
    title: '팀 소개 | Braillify',
    description:
      'Braillify를 만들어가는 팀원들을 소개합니다. 데브파이브와 오픈소스 기여자들이 함께 한국 점자 변환 라이브러리를 개발합니다.',
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
}

export default function TeamPage() {
  return (
    <VStack
      flex="1"
      gap="40px"
      maxW="1520px"
      minH="calc(100vh - 100px)"
      mx="auto"
      px={['16px', null, '30px', '60px']}
      py={['30px', null, null, '40px']}
      w="100%"
    >
      <VStack gap="20px">
        <Text as="h1" color="$title" m="0" typography="docsTitle">
          팀 소개
        </Text>
        <Text color="$text" typography="body">
          Braillify 를 주도하는 팀원들입니다.
        </Text>
      </VStack>
      <Box bg="$text" h="1px" />
      <Flex w="100%">
        <Flex flexWrap="wrap" gap="20px" px={[null, null, null, '20px']}>
          <TeamMemberCard
            bgImage="/images/team/image-01.jpg"
            githubUrl="https://github.com/owjs3901"
            instagramUrl="https://www.instagram.com/owjs3901"
            name="Jeong Min Oh"
            position="LEAD"
            profileImage="https://avatars.githubusercontent.com/u/12480623?v=4"
          />
        </Flex>
      </Flex>
    </VStack>
  )
}
