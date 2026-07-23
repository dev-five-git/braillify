'use client'

import { Box, Center, Flex, Text, VStack } from '@devup-ui/react'
import Link from 'next/link'
import { usePathname, useRouter, useSearchParams } from 'next/navigation'
import { useEffect, useState } from 'react'

import { isSurveyActive, SURVEY_URL } from '@/constants/survey'

import MobileMenuProvider, {
  MobileMenuItem,
  MobileMenuItemBox,
  MobileMenuSubItem,
} from './MobileMenuProvider'
import ThemeSwitch from './ThemeSwitch'

export default function MobileMenu() {
  const router = useRouter()
  const searchParams = useSearchParams()
  const menuOpen = searchParams.get('menuOpen')
  const pathname = usePathname()
  const [showSurvey, setShowSurvey] = useState(false)

  useEffect(() => {
    setShowSurvey(isSurveyActive())
  }, [])

  return (
    <>
      {menuOpen === 'true' && (
        <VStack
          bg="$containerBackground"
          color="$text"
          h="100vh"
          inset="0"
          position="fixed"
          w="100%"
          zIndex="200"
        >
          <Flex
            borderBottom="1px solid $border"
            justifyContent="space-between"
            w="100%"
          >
            <Flex alignItems="center">
              <Link
                aria-label="GitHub link"
                href="https://github.com/dev-five-git/braillify"
                target="_blank"
              >
                <Center px="10px" py="13px">
                  <Box
                    bg="$text"
                    boxSize="24px"
                    h={['32px', null, null, '50px']}
                    maskImage="url(/images/github.svg)"
                    maskPosition="center"
                    maskRepeat="no-repeat"
                    maskSize="contain"
                    zIndex="1"
                  />
                </Center>
              </Link>
              <Link
                aria-label="Discord link"
                href="https://discord.gg/8zjcGc7cWh"
                target="_blank"
              >
                <Center px="10px" py="13px">
                  <Box
                    bg="$text"
                    boxSize="24px"
                    h={['32px', null, null, '50px']}
                    maskImage="url(/images/discord.svg)"
                    maskPosition="center"
                    maskRepeat="no-repeat"
                    maskSize="contain"
                    zIndex="1"
                  />
                </Center>
              </Link>
              <Link
                aria-label="Kakao Open Chat link"
                href="https://open.kakao.com/o/gzeq4eBh"
                target="_blank"
              >
                <Center px="10px" py="13px">
                  <Box
                    bg="$text"
                    boxSize="24px"
                    h={['32px', null, null, '50px']}
                    maskImage="url(/images/kakao.svg)"
                    maskPosition="center"
                    maskRepeat="no-repeat"
                    maskSize="contain"
                    zIndex="1"
                  />
                </Center>
              </Link>
              <Center px="10px" py="13px">
                <ThemeSwitch />
              </Center>
            </Flex>
            <Center
              cursor="pointer"
              onClick={() => router.push(`?menuOpen=false`)}
              p="10px"
            >
              <Box
                bg="$text"
                boxSize="32px"
                maskImage="url(/images/close.svg)"
                maskPosition="center"
                maskRepeat="no-repeat"
                maskSize="contain"
              />
            </Center>
          </Flex>
          <MobileMenuProvider>
            <MobileMenuItem>
              <Box borderBottom="1px solid $border" px="16px" py="20px">
                <Text typography="button">문서</Text>
              </Box>
            </MobileMenuItem>
            <MobileMenuSubItem>
              <VStack bg="$background" pt="4px" px="16px" typography="docsMenu">
                <MobileMenuItemBox
                  onClick={() => router.push(`/docs/overview`)}
                  selected={pathname === '/docs/overview'}
                >
                  <Flex
                    alignItems="center"
                    gap="10px"
                    justifyContent="space-between"
                    w="100%"
                  >
                    <Text>개요</Text>
                  </Flex>
                </MobileMenuItemBox>
                <MobileMenuItemBox
                  onClick={() => router.push(`/docs/installation`)}
                  selected={pathname === '/docs/installation'}
                >
                  <Flex
                    alignItems="center"
                    gap="10px"
                    justifyContent="space-between"
                    w="100%"
                  >
                    <Text>설치</Text>
                  </Flex>
                </MobileMenuItemBox>
                <MobileMenuItemBox
                  onClick={() => router.push(`/docs/api`)}
                  selected={pathname === '/docs/api'}
                >
                  <Flex
                    alignItems="center"
                    gap="10px"
                    justifyContent="space-between"
                    w="100%"
                  >
                    <Text>API</Text>
                  </Flex>
                </MobileMenuItemBox>
              </VStack>
            </MobileMenuSubItem>
          </MobileMenuProvider>
          <Box
            borderBottom="1px solid $border"
            cursor="pointer"
            onClick={() => router.push(`/test-case`)}
            px="16px"
            py="20px"
          >
            <Text typography="button">테스트 케이스</Text>
          </Box>
          <Box
            borderBottom="1px solid $border"
            cursor="pointer"
            onClick={() => router.push(`/team`)}
            px="16px"
            py="20px"
          >
            <Text typography="button">팀 소개</Text>
          </Box>
          {showSurvey && (
            <Box
              aria-label="Survey link"
              as="a"
              borderBottom="1px solid $border"
              cursor="pointer"
              display="block"
              href={SURVEY_URL}
              px="16px"
              py="20px"
              rel="noopener noreferrer"
              target="_blank"
            >
              <Text typography="button">설문 참여하기</Text>
            </Box>
          )}
        </VStack>
      )}
    </>
  )
}
