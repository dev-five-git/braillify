'use client'

import { Box, Center, Flex, Text, VStack } from '@devup-ui/react'
import Link from 'next/link'
import { useEffect, useRef, useState } from 'react'

import {
  Controls as Choice,
  field,
  fieldLabel,
  primaryAction,
  retryAction,
  secondaryAction,
} from '@/components/together/Controls'
import { loadTogetherList, type Partner } from '@/lib/together-list'
import { previewDescription } from '@/lib/together-preview'

const kinds = ['기업', '기관', '학교', '단체']

export default function Together() {
  const [kind, setKind] = useState('기업')
  const [name, setName] = useState('')
  const [braille, setBraille] = useState('')
  const [partners, setPartners] = useState<Partner[] | null>(null)
  const [failed, setFailed] = useState(false)
  const [attempt, setAttempt] = useState(0)
  const [visible, setVisible] = useState(60)
  const [announcement, setAnnouncement] = useState('')
  const focusId = useRef<number | null>(null)

  useEffect(() => {
    let cancelled = false
    const timeout = setTimeout(() => {
      import('braillify')
        .then(
          (mod) =>
            !cancelled &&
            setBraille(
              mod.translateToUnicode(name || '이름을 입력하면 여기에 표시돼요'),
            ),
        )
        .catch(() => {
          if (!cancelled) setBraille('')
        })
    }, 200)
    return () => {
      cancelled = true
      clearTimeout(timeout)
    }
  }, [name])

  useEffect(() => {
    const controller = new AbortController()
    let active = true
    loadTogetherList(controller.signal)
      .then((data: Partner[]) => {
        if (!active) return
        setPartners(data)
        const id = /^#org-(\d+)$/.exec(window.location.hash)?.[1]
        const index = data.findIndex((item) => String(item.issue) === id)
        if (index >= 0) {
          focusId.current = data[index].issue
          setVisible(Math.ceil((index + 1) / 60) * 60)
        }
      })
      .catch(() => {
        if (active) setFailed(true)
      })
    return () => {
      active = false
      controller.abort()
    }
  }, [attempt])

  useEffect(() => {
    if (focusId.current === null) return
    const element = document.getElementById(`org-${focusId.current}`)
    if (element) {
      element.focus()
      element.scrollIntoView({
        block: 'center',
        behavior: window.matchMedia('(prefers-reduced-motion: reduce)').matches
          ? 'instant'
          : 'smooth',
      })
      focusId.current = null
    }
  }, [partners, visible])

  const register = `/together/register?${new URLSearchParams({ kind, name })}`
  return (
    <VStack
      gap="40px"
      maxW="1520px"
      mx="auto"
      px={['16px', null, '30px', '60px']}
      py="40px"
      w="100%"
    >
      <VStack gap="20px">
        <Text as="h1" color="$title" m="0" typography="docsTitle">
          Braillify와 함께하는 곳
        </Text>
        <Text color="$text" typography="body">
          Braillify를 쓰고 있는 기업·기관·학교·단체를 소개합니다. 신청하면 실제
          사용을 확인한 뒤 이곳에 함께 소개해 드려요.
        </Text>
      </VStack>
      <VStack gap="20px">
        <Flex
          alignItems={['stretch', null, null, 'center']}
          flexDir={['column', null, null, 'row']}
          gap={['24px', null, null, '64px']}
        >
          <VStack
            alignItems="center"
            aria-hidden="true"
            bg="$containerBackground"
            border="2px solid $primary"
            borderRadius={['12px', null, null, '16px']}
            gap={['16px', null, null, '32px']}
            maxW="680px"
            pb={['20px', null, null, '40px']}
            position="relative"
            pt={['8px', null, null, '40px']}
            px={['8px', null, null, '48px']}
            w="100%"
          >
            <Flex
              alignSelf="flex-start"
              bg="$primary"
              borderRadius="999px"
              left={['auto', null, null, '18px']}
              position={['static', null, null, 'absolute']}
              px={['10px', null, null, '12px']}
              py={['4px', null, null, '5px']}
              top={['auto', null, null, '18px']}
            >
              <Text color="$base" typography="tinyBtn">
                미리 보기
              </Text>
            </Flex>
            <Flex
              alignItems="center"
              gap={['12px', null, null, '20px']}
              pb={['4px', null, null, '8px']}
              w="100%"
            >
              <Flex flex="1" justifyContent="flex-end">
                <Box
                  aspectRatio="838/363"
                  bg="$text"
                  h={['32px', null, null, '52px']}
                  maskImage="url(/images/home/hero.svg)"
                  maskPosition="center"
                  maskRepeat="no-repeat"
                  maskSize="contain"
                />
              </Flex>
              <Box
                bg="$caption"
                boxSize={['12px', null, null, '16px']}
                maskImage="url(/images/together/cross.svg)"
                maskPosition="center"
                maskRepeat="no-repeat"
                maskSize="contain"
              />
              <Flex flex="1">
                <Center
                  bg="$background"
                  borderRadius="8px"
                  h={['28px', null, null, '48px']}
                  w={['72px', null, null, '120px']}
                >
                  <Text color="$caption" typography="docsCaption">
                    로고
                  </Text>
                </Center>
              </Flex>
            </Flex>
            <VStack
              alignItems="center"
              gap={['8px', null, null, '12px']}
              w="100%"
            >
              <Text
                color={name ? '$title' : '$caption'}
                fontSize={['18px', null, null, '32px']}
                fontWeight="700"
                letterSpacing="-0.06em"
                lineHeight={['24px', null, null, '40px']}
                textAlign="center"
                wordBreak="keep-all"
              >
                {name || '이름을 입력하면 여기에 표시돼요'}
              </Text>
              <Text
                color={name ? '$caption' : '#B5B3AE'}
                fontSize={['13px', null, null, '16px']}
                lineHeight={['13px', null, null, '16px']}
                textAlign="center"
                wordBreak="break-all"
              >
                {braille}
              </Text>
            </VStack>
            <Box bg="$primary" h="2px" w={['32px', null, null, '40px']} />
            <VStack alignItems="center" gap={['8px', null, null, '12px']}>
              <Text
                color="$text"
                display={['none', null, null, 'block']}
                textAlign="center"
                typography="body"
              >
                위 {kind}
                {kind === '기업' || kind === '기관' ? '은' : '는'} Braillify와
                함께하는 {kind}입니다.
              </Text>
              <Text
                color="$text"
                display={['block', null, null, 'none']}
                textAlign="center"
                typography="docsCaption"
              >
                위 {kind}
                {kind === '기업' || kind === '기관' ? '은' : '는'} Braillify와
                함께하는 {kind}입니다.
              </Text>
              <Text
                _themeDark={{ color: '$caption' }}
                color="#666"
                display={['none', null, null, 'block']}
                textAlign="center"
                typography="docsSmall"
              >
                braillify.kr/together · 승인 연월
              </Text>
              <Text
                _themeDark={{ color: '$caption' }}
                color="#666"
                display={['block', null, null, 'none']}
                textAlign="center"
                typography="footer"
              >
                braillify.kr/together · 승인 연월
              </Text>
            </VStack>
          </VStack>
          <Box
            aria-live="polite"
            border="0"
            clip="rect(0, 0, 0, 0)"
            height="1px"
            margin="-1px"
            overflow="hidden"
            padding="0"
            position="absolute"
            role="status"
            whiteSpace="nowrap"
            width="1px"
          >
            {previewDescription(name, kind, braille)}
          </Box>
          <VStack flex="1" gap="24px">
            <VStack gap="2px">
              <Text as="h2" color="$title" typography="featureTitle">
                우리 이름으로 미리 보기
              </Text>
              <Text color="$text" typography="body">
                승인되면 이 인증 이미지를 GitHub 답글로 보내 드려요.
              </Text>
            </VStack>
            <Box as="fieldset" border="none">
              <Text as="legend" color="$title" mb="8px" typography="bodyBold">
                구분
              </Text>
              <Flex flexWrap="wrap" gap="8px">
                {kinds.map((value) => (
                  <Choice
                    key={value}
                    checked={kind === value}
                    name="preview-kind"
                    onChange={() => setKind(value)}
                    type="radio"
                  >
                    {value}
                  </Choice>
                ))}
              </Flex>
            </Box>
            <label className={fieldLabel} htmlFor="preview-name">
              이름 · 로고는 신청할 때 GitHub에서 올려요.
              <input
                className={field}
                id="preview-name"
                maxLength={20}
                onChange={(e) => setName(e.target.value)}
                placeholder="예: 점자 교육 연구회"
                value={name}
              />
            </label>
            <Link className={primaryAction} href={register}>
              이 이름으로 신청하기
              <Box
                aria-hidden="true"
                bg="$base"
                boxSize="20px"
                maskImage="url(/images/together/action-dot.svg)"
                maskPosition="center"
                maskRepeat="no-repeat"
                maskSize="contain"
                ml="12px"
              />
            </Link>
          </VStack>
        </Flex>
      </VStack>
      <Box aria-hidden="true" bg="$primary" h="1px" />
      <VStack gap="20px">
        <Text as="h2" color="$title" typography="featureTitle">
          함께하는 곳
        </Text>
        {partners && partners.length > 10 && (
          <Text color="$text">지금 {partners.length}곳이 함께하고 있어요</Text>
        )}
        {failed ? (
          <VStack alignItems="center" gap="8px">
            <Center
              bg="$menuHover"
              borderRadius="16px"
              flexDir="column"
              gap={['10px', null, null, '14px']}
              minH={['135px', null, null, '155px']}
              px={['16px', null, null, '24px']}
              py={['24px', null, null, '32px']}
              role="status"
              w="100%"
            >
              <Box
                aria-hidden="true"
                bg="$text"
                boxSize={['22px', null, null, '24px']}
                maskImage="url(/images/together/alert.svg)"
                maskPosition="center"
                maskRepeat="no-repeat"
                maskSize="contain"
              />
              <Text color="$text" textAlign="center" typography="bodyLg">
                목록을 불러오지 못했어요.
                <Box as="br" display={['block', null, null, 'none']} /> 잠시 뒤
                다시 확인해 주세요.
              </Text>
            </Center>
            <button
              className={retryAction}
              onClick={() => {
                setFailed(false)
                setPartners(null)
                setAttempt((value) => value + 1)
              }}
              type="button"
            >
              다시 시도
            </button>
          </VStack>
        ) : !partners ? (
          <Center
            aria-label="목록을 불러오는 중"
            bg="$menuHover"
            borderRadius="16px"
            minH={['135px', null, null, '155px']}
            role="status"
          />
        ) : partners.length === 0 ? (
          <Center
            bg="$menuHover"
            borderRadius="16px"
            flexDir="column"
            gap="12px"
            minH={['135px', null, null, '155px']}
          >
            <Text aria-hidden="true" color="$caption">
              ⠿
            </Text>
            <Text
              color="$title"
              fontSize={['14px', null, null, '18px']}
              fontWeight="700"
            >
              첫 번째로 함께해 주세요
            </Text>
          </Center>
        ) : (
          <Flex
            flexWrap="wrap"
            gap={['12px', null, null, '20px']}
            justifyContent="center"
          >
            {partners.slice(0, visible).map((item) => (
              <VStack
                key={item.issue}
                _target={{ borderWidth: '2px' }}
                alignItems="center"
                alignSelf="stretch"
                aria-label={item.name}
                bg="$containerBackground"
                border="1px solid $primary"
                borderRadius="16px"
                gap={['8px', null, null, '12px']}
                id={`org-${item.issue}`}
                pb={['16px', null, null, '20px']}
                position="relative"
                pt={['28px', null, null, '20px']}
                px={['12px', null, null, '16px']}
                tabIndex={-1}
                w={['158px', null, null, '216px']}
              >
                <Flex
                  bg="$menuHover"
                  borderRadius="6px"
                  left={['7px', null, null, '11px']}
                  position="absolute"
                  px={['6px', null, null, '8px']}
                  py={['2px', null, null, '3px']}
                  top={['7px', null, null, '11px']}
                >
                  <Text color="$text" typography="tinyBtn">
                    {item.kind}
                  </Text>
                </Flex>
                <Center h={['56px', null, null, '72px']} w="100%">
                  <img
                    alt=""
                    height="60"
                    src={item.logo}
                    style={{
                      objectFit: 'contain',
                      maxWidth: '100%',
                      maxHeight: '100%',
                    }}
                    width="100"
                  />
                </Center>
                <Text
                  color="$title"
                  display="-webkit-box"
                  overflow="hidden"
                  textAlign="center"
                  textOverflow="ellipsis"
                  typography="bodyBold"
                  w="100%"
                  WebkitBoxOrient="vertical"
                  WebkitLineClamp="2"
                  wordBreak="keep-all"
                >
                  {item.name}
                </Text>
              </VStack>
            ))}
          </Flex>
        )}
        <span aria-live="polite">{announcement}</span>
        {partners && visible < partners.length && (
          <button
            className={secondaryAction}
            onClick={() => {
              const count = Math.min(60, partners.length - visible)
              focusId.current = partners[visible].issue
              setVisible(visible + count)
              setAnnouncement(`${count}곳을 더 보여 드려요`)
            }}
          >
            더 보기 ({visible}/{partners.length})
          </button>
        )}
      </VStack>
      <VStack gap="24px" w="100%">
        <Box aria-hidden="true" bg="$primary" h="1px" />
        <Text color="$text" typography="body" wordBreak="keep-all">
          게시를 멈추고 싶으면 contact@devfive.kr로 알려 주세요.
        </Text>
      </VStack>
    </VStack>
  )
}
