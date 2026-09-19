'use client'

import { Box, Center, Flex } from '@devup-ui/react'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { Suspense, useRef } from 'react'

import { useIntersectionObserver } from '@/hooks/useIntersectionObserver'

import IconHamburger from '../icons/IconHamburger'
import MobileMenu from './MobileMenu'
import MobileMenuButton from './MobileMenuButton'
import Pages from './Pages'
import ThemeSwitch from './ThemeSwitch'

export default function Header() {
  const headerRef = useRef<HTMLDivElement>(null)
  const isIntersecting = useIntersectionObserver(
    headerRef,
    {
      rootMargin: '-50px',
      threshold: 0,
    },
    true,
  )

  const pathname = usePathname()

  const isIntersectingHome = pathname === '/' && isIntersecting
  const isTestCase = pathname === '/test-case'

  return (
    <>
      <Box
        as="header"
        bg={isTestCase ? '$background' : 'transparent'}
        h={['60px', null, null, '100px']}
        left="0"
        p={['4px', null, null, '10px']}
        position="sticky"
        right="0"
        top="0"
        w="100%"
        zIndex="100"
      >
        <Flex
          alignItems="center"
          bg={[
            isIntersectingHome ? 'transparent' : '$containerBackground',
            null,
            null,
            isIntersectingHome ? 'transparent' : '$containerBackground',
          ]}
          borderRadius={['10px', null, null, '20px']}
          flex="1"
          h="100%"
          justifyContent="space-between"
          position="relative"
          px={['16px', null, null, '40px']}
          transition="background-color 0.3s ease"
        >
          <Link
            aria-label="Home link"
            href={isIntersectingHome ? '#' : '/'}
            onClick={(e) => {
              if (!isIntersectingHome && pathname === '/') {
                e.preventDefault()
              }
            }}
            style={{ cursor: isIntersectingHome ? 'default' : 'pointer' }}
          >
            <Box
              aspectRatio="838/363"
              bg="$text"
              h={['32px', null, null, '50px']}
              maskImage="url(/images/home/hero.svg)"
              maskSize="contain"
              onClick={() => {
                if (!isIntersectingHome && pathname === '/') {
                  window.scrollTo({ top: 0, behavior: 'smooth' })
                }
              }}
              opacity={[
                isIntersectingHome ? 0 : 1,
                null,
                null,
                isIntersectingHome ? 0 : 1,
              ]}
              position="relative"
              transition="opacity 0.3s ease"
              zIndex="1"
            />
          </Link>
          <Pages isIntersecting={isIntersectingHome} />
          <Flex
            alignItems="center"
            display={['none', null, null, 'flex']}
            gap="32px"
          >
            <Link
              aria-label="GitHub link"
              href="https://github.com/dev-five-git/braillify"
              target="_blank"
            >
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
            </Link>
            <Link
              aria-label="Discord link"
              href="https://discord.gg/8zjcGc7cWh"
              target="_blank"
            >
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
            </Link>
            <Link
              aria-label="Kakao Open Chat link"
              href="https://open.kakao.com/o/gzeq4eBh"
              target="_blank"
            >
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
            </Link>
            <ThemeSwitch />
          </Flex>
          <Flex
            alignItems="center"
            display={['flex', null, null, 'none']}
            gap="10px"
          >
            <Suspense>
              <MobileMenuButton>
                <Center>
                  <IconHamburger />
                </Center>
              </MobileMenuButton>
            </Suspense>
          </Flex>
        </Flex>
      </Box>
      <Box ref={headerRef} h="0" />
      <Suspense>
        <MobileMenu />
      </Suspense>
    </>
  )
}
