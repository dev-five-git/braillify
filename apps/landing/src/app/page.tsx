import { Box, Center, Flex, Image, Text, VStack } from '@devup-ui/react'

import HomePopup from '@/components/home/HomePopup'
import PillButton from '@/components/home/PillButton'
import IconDiscord from '@/components/icons/IconDiscord'
import IconKakao from '@/components/icons/IconKakao'

import { DemoTabs } from './DemoTabs'

const SITE_URL = 'https://braillify.kr'

// Page-level JSON-LD: Breadcrumb + FAQ for rich results
const pageJsonLd = [
  {
    '@context': 'https://schema.org',
    '@type': 'BreadcrumbList',
    itemListElement: [
      {
        '@type': 'ListItem',
        position: 1,
        name: '홈',
        item: SITE_URL,
      },
    ],
  },
  {
    '@context': 'https://schema.org',
    '@type': 'FAQPage',
    mainEntity: [
      {
        '@type': 'Question',
        name: 'Braillify(브레일리파이)는 무엇인가요?',
        acceptedAnswer: {
          '@type': 'Answer',
          text: 'Braillify는 2024 개정 한국 점자 규정을 기반으로 설계된 오픈소스 한국어 점자 변환 라이브러리입니다. Rust로 개발되어 Node.js, Python, Rust, WebAssembly 환경에서 실시간 점역(점자 변환)을 지원합니다.',
        },
      },
      {
        '@type': 'Question',
        name: 'Braillify는 어떤 언어/플랫폼을 지원하나요?',
        acceptedAnswer: {
          '@type': 'Answer',
          text: 'Node.js(npm), Python(PyPI), Rust(crates.io)에서 설치 가능하며 WebAssembly(wasm)로도 브라우저에서 바로 실행할 수 있습니다. 네트워크 연결 없이 로컬에서 점자 변환이 가능합니다.',
        },
      },
      {
        '@type': 'Question',
        name: '기존 점역기(점사랑, 하상브레일 등)와 무엇이 다른가요?',
        acceptedAnswer: {
          '@type': 'Answer',
          text: 'Braillify는 완전 오픈소스(Apache License 2.0)로 공개되어 누구나 코드를 검증하고 개선할 수 있습니다. 2024 개정 한국 점자 규정을 기반으로 문맥을 고려한 자연스러운 점역 결과를 제공합니다.',
        },
      },
      {
        '@type': 'Question',
        name: '점자 번역과 점역의 차이는 무엇인가요?',
        acceptedAnswer: {
          '@type': 'Answer',
          text: '일반적으로 검색에는 "점자 번역"이라는 용어가 많이 사용되지만, 정확한 표현은 "점역(點譯)"입니다. Braillify는 한국어 텍스트를 점자로 변환하는 점역 라이브러리입니다.',
        },
      },
    ],
  },
]

const DESCRIPTIONS = [
  {
    title: '2024 개정 한국 점자 규정 기반 점역기',
    description:
      'braillify는 2024년 개정된 한국 점자 규정을 기반으로 설계되고 구현된 점역기입니다.더 이상 유지보수가 어렵고, 레거시 코드에 의존해 최신 규정과 맞지 않는 점역기를 사용할 필요가 없습니다. 글의 문맥을 고려해 다양한 경우의 수를 판단하여 더욱 자연스럽고 정확한 점역 결과를 제공합니다.',
  },
  {
    title: '완전한 오픈소스 프로젝트',
    description:
      '기존에도 점사랑, 하상브레일 등 다양한 점역기가 존재했고, 일부는 API를 제공하기도 했습니다.하지만 이들은 대부분 소스가 공개되지 않았고, 점역을 위해 API 서버에 연결해야 했습니다. braillify는 다릅니다. 누구나 접근하고, 함께 개선해 나갈 수 있도록 점자 표준 구현 전 과정을 오픈소스로 제공합니다.',
  },
  {
    title: 'Rust 기반 크로스 플랫폼',
    description:
      'braillify는 Rust 언어로 개발되었으며, Node.js, Rust, Python 환경을 모두 지원합니다. 또한 WebAssembly(wasm)도 지원하여, 네트워크나 외부 연결 없이 자신의 PC에서 바로 실행 가능한 구조를 가지고 있습니다. 이를 통해 플랫폼에 구애받지 않고 더 자유롭고 유연한 활용이 가능합니다. 원하는 플랫폼이 있다면 Devfive와 함께 braillify를 확장하고 발전시켜보세요.',
  },
]

export default function HomePage() {
  return (
    <VStack alignItems="center" bg="$background" position="relative">
      <script
        dangerouslySetInnerHTML={{ __html: JSON.stringify(pageJsonLd) }}
        type="application/ld+json"
      />
      <HomePopup />
      <Box
        px={['16px', null, '30px', '80px']}
        py={['40px', null, null, '100px']}
        w="100%"
      >
        <Image
          alt=""
          aria-hidden="true"
          display={['none', null, null, null, 'block']}
          h="1019px"
          pos="absolute"
          right="47.172px"
          role="presentation"
          src="/images/home/background-braille.svg"
          top="145px"
          w="236px"
        />
        <VStack
          alignItems="center"
          gap="80px"
          position="relative"
          w="100%"
          zIndex="1"
        >
          <VStack alignItems={['center', null, null, 'flex-start']} w="100%">
            {/*
              The hero SVG (Braillify wordmark) acts as the visual H1.
              The <h1> wraps the mask-rendered logo AND a visually-hidden
              text label so SEO crawlers and screen readers receive the
              actual heading text "Braillify - 한국어 점자 변환 라이브러리".
            */}
            <Box
              as="h1"
              m="0"
              maxW="1000px"
              mb={['30px', null, null, '60px']}
              position="relative"
              w={['100%', null, null, '60%']}
            >
              <Box
                aria-hidden="true"
                aspectRatio="838/341"
                bg="$text"
                maskImage="url(/images/home/hero.svg)"
                maskPosition="start"
                maskRepeat="no-repeat"
                maskSize="contain"
                w="100%"
              />
              <Box
                border="0"
                clipPath="inset(50%)"
                h="1px"
                m="-1px"
                overflow="hidden"
                p="0"
                position="absolute"
                w="1px"
                whiteSpace="nowrap"
              >
                Braillify - 한국어 점자 변환 라이브러리
              </Box>
            </Box>
            <VStack
              alignItems={['center', null, null, 'flex-start']}
              gap={['20px', null, null, '40px']}
              w="100%"
            >
              <Text as="h2" color="$text" m="0" typography="mainText">
                실시간 한글 점역 라이브러리
              </Text>
              <PillButton aria-label="Start now button" href="/docs/overview">
                <Text color="#FFF" typography="buttonLg">
                  문서 둘러보기
                </Text>
              </PillButton>
            </VStack>
          </VStack>
          <Flex maxW="1520px" w="100%">
            <DemoTabs />
          </Flex>
        </VStack>
      </Box>
      <Flex
        flexDirection={['column', null, null, 'row']}
        gap="80px"
        maxW="1640px"
        mx="auto"
        px={['16px', null, null, '60px']}
        py={['30px', null, null, '80px']}
        wordBreak="keep-all"
      >
        <VStack gap="20px">
          <Flex gap="16px">
            <Text as="h2" color="$text" m="0" typography="title">
              braillify의 특징
            </Text>
            <Box aspectRatio="1" bg="$text" borderRadius="50%" h="16px" />
          </Flex>
          <Text color="$text" typography="bodyLg">
            ‘Braille(점자)’에 ‘-ify(~화化하다)’를 더해
            <br /> 한층 더 쉬운 점자화를 보다 널리 퍼뜨리고자 만든
            프로젝트입니다.
            <br />
            모두가 점역을 이해하고 활용할 수 있는 환경을 함께 만들어갑니다.
          </Text>
        </VStack>
        <VStack flex="1" justifyContent="center">
          {DESCRIPTIONS.map(({ title, description }, index) => (
            <Flex
              key={`description-${index}`}
              borderBottom="1px solid $text"
              borderTop="1px solid $text"
              gap="50px"
              p={['16px', null, null, '50px']}
              position="relative"
            >
              <VStack flex="1" gap="12px">
                <Text color="$text" typography="featureCount">
                  {(index + 1).toString().padStart(2, '0')}
                </Text>
                <Text as="h3" color="$text" m="0" typography="featureTitle">
                  {title}
                </Text>
                <Text color="$text" typography="body">
                  {description}
                </Text>
              </VStack>
              <Box
                aspectRatio="1"
                bg="$text"
                borderRadius="50%"
                boxSize={['12px', null, null, '16px']}
                position={['absolute', null, null, 'static']}
                right="20px"
                top="20px"
              />
            </Flex>
          ))}
        </VStack>
      </Flex>
      <Center gap="100px" px={['16px', null, null, '80px']} py="100px" w="100%">
        <Flex
          alignItems={['center', null, null, 'flex-start']}
          bg="url(/images/home/texture.png)"
          bgPosition="center"
          bgSize="cover"
          borderRadius={['20px', null, null, '40px']}
          flex="1"
          flexDirection={['column', null, null, 'row']}
          justifyContent="space-between"
          maxW="1520px"
          position="relative"
          px={['50px', null, null, '100px']}
          py={['30px', null, null, '80px']}
          w="100%"
        >
          <VStack gap="20px">
            <Flex
              gap="16px"
              justifyContent={['center', null, null, 'flex-start']}
            >
              <Text
                as="h2"
                color="#FFF"
                m="0"
                typography="title"
                whiteSpace="nowrap"
              >
                공식 커뮤니티 참여하기
              </Text>
              <Box aspectRatio="1" bg="#FFF" borderRadius="50%" h="16px" />
            </Flex>
            <Text
              color="#FFF"
              textAlign={['center', null, null, 'left']}
              typography="bodyLg"
            >
              braillify의 커뮤니티에 참여해
              <br /> 점자와 세상, 모두를 연결하는 여정을 시작해보세요.
            </Text>
          </VStack>
          <VStack gap="20px" justifyContent="center" pt="80px">
            <PillButton
              aria-label="Kakao Open Chat button"
              href="https://open.kakao.com/o/gzeq4eBh"
              target="_blank"
            >
              <Flex gap="16px">
                <IconKakao color="#FFF" />
                <Text color="#FFF" typography="button">
                  카카오톡 오픈 채팅
                </Text>
              </Flex>
            </PillButton>
            <PillButton
              aria-label="Discord server button"
              href="https://discord.gg/8zjcGc7cWh"
              target="_blank"
            >
              <Flex gap="16px">
                <IconDiscord color="#FFF" />
                <Text color="#FFF" typography="button">
                  디스코드 서버
                </Text>
              </Flex>
            </PillButton>
          </VStack>
        </Flex>
      </Center>
    </VStack>
  )
}
