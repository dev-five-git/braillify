'use client'

import { Box, Flex, Text, VStack } from '@devup-ui/react'
import Link from 'next/link'
import { useEffect, useRef, useState } from 'react'

type Partner = { issue: number; kind: string; name: string; logo: string }
const kinds = ['기업', '기관', '학교', '단체']

export default function Together() {
  const [kind, setKind] = useState('기업')
  const [name, setName] = useState('')
  const [braille, setBraille] = useState('')
  const [partners, setPartners] = useState<Partner[] | null>(null)
  const [failed, setFailed] = useState(false)
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
    fetch('/together.json')
      .then((response) => {
        if (!response.ok) throw new Error('Failed to load')
        return response.json()
      })
      .then((data: Partner[]) => {
        setPartners(data)
        const id = /^#org-(\d+)$/.exec(window.location.hash)?.[1]
        const index = data.findIndex((item) => String(item.issue) === id)
        if (index >= 0) {
          focusId.current = data[index].issue
          setVisible(Math.ceil((index + 1) / 60) * 60)
        }
      })
      .catch(() => setFailed(true))
  }, [])

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
          함께하는 곳
        </Text>
        <Text color="$text" typography="body">
          신청하면 실제 사용을 확인한 뒤 이곳에 함께 소개해 드려요.
        </Text>
      </VStack>
      <VStack gap="20px">
        <Text as="h2" color="$title" typography="featureTitle">
          인증 이미지 미리 보기
        </Text>
        <Text color="$text">
          승인되면 이 인증 이미지를 GitHub 답글로 보내 드려요.
        </Text>
        <Flex flexWrap="wrap" gap="32px">
          <VStack
            aria-hidden="true"
            bg="$containerBackground"
            border="2px solid $primary"
            borderRadius="16px"
            gap="32px"
            maxW="680px"
            p={['20px', null, null, '40px']}
            w="100%"
          >
            <Text color="$caption">미리 보기</Text>
            <Text color="$title" typography="featureTitle">
              braillify × 로고
            </Text>
            <Text color="$title" typography="featureTitle">
              {name || '이름을 입력하면 여기에 표시돼요'}
            </Text>
            <Text color="$caption">{braille}</Text>
            <Box bg="$border" h="1px" />
            <Text color="$text">
              위 {kind}
              {kind === '기업' || kind === '기관' ? '은' : '는'} Braillify와
              함께하는 {kind}입니다.
            </Text>
            <Text color="$caption">braillify.kr/together</Text>
          </VStack>
          <VStack gap="16px">
            <label htmlFor="preview-kind">구분</label>
            <select
              id="preview-kind"
              onChange={(e) => setKind(e.target.value)}
              value={kind}
            >
              {kinds.map((value) => (
                <option key={value}>{value}</option>
              ))}
            </select>
            <label htmlFor="preview-name">
              이름 · 로고는 신청할 때 GitHub에서 올려요.
            </label>
            <input
              id="preview-name"
              maxLength={20}
              onChange={(e) => setName(e.target.value)}
              value={name}
            />
            <Link href={register}>이 이름으로 신청하기</Link>
          </VStack>
        </Flex>
      </VStack>
      <Box bg="$border" h="1px" />
      <VStack gap="20px">
        <Text as="h2" color="$title" typography="featureTitle">
          함께하는 곳
        </Text>
        {partners && partners.length > 10 && (
          <Text color="$text">지금 {partners.length}곳이 함께하고 있어요</Text>
        )}
        {failed ? (
          <Text color="$text">
            목록을 불러오지 못했어요. 새로고침해 주세요.
          </Text>
        ) : !partners ? (
          <Text color="$text">목록을 불러오는 중이에요.</Text>
        ) : partners.length === 0 ? (
          <Text color="$text">첫 번째로 함께해 주세요.</Text>
        ) : (
          <Flex flexWrap="wrap" gap="20px" justifyContent="center">
            {partners.slice(0, visible).map((item) => (
              <VStack
                key={item.issue}
                _target={{ borderWidth: '2px' }}
                aria-label={item.name}
                bg="$containerBackground"
                border="1px solid $primary"
                borderRadius="16px"
                gap="12px"
                id={`org-${item.issue}`}
                p="20px"
                tabIndex={-1}
                w={['158px', null, null, '216px']}
              >
                <Text color="$text">{item.kind}</Text>
                <img
                  alt=""
                  height="60"
                  src={item.logo}
                  style={{ objectFit: 'contain' }}
                  width="100"
                />
                <Text color="$title">{item.name}</Text>
              </VStack>
            ))}
          </Flex>
        )}
        <span aria-live="polite">{announcement}</span>
        {partners && visible < partners.length && (
          <button
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
      <Text color="$text">
        게시를 멈추고 싶으면 contact@devfive.kr로 알려 주세요.
      </Text>
    </VStack>
  )
}
