'use client'

import { Box, Flex, Image, Text, VStack } from '@devup-ui/react'
import { useEffect, useState } from 'react'

import { isSurveyActive, SURVEY_URL } from '@/constants/survey'

const POPUP_STORAGE_KEY = 'braillify-popup-hide-until'

export default function HomePopup() {
  const [open, setOpen] = useState(false)

  useEffect(() => {
    if (!isSurveyActive()) return
    const hideUntil = localStorage.getItem(POPUP_STORAGE_KEY)
    if (hideUntil && Date.now() < Number(hideUntil)) return
    setOpen(true)
  }, [])

  if (!open) return null

  const handleHideForToday = () => {
    const tomorrow = new Date()
    tomorrow.setHours(24, 0, 0, 0)
    localStorage.setItem(POPUP_STORAGE_KEY, String(tomorrow.getTime()))
    setOpen(false)
  }

  const handleClose = () => setOpen(false)

  return (
    <Flex
      alignItems="center"
      bg="rgba(0, 0, 0, 0.5)"
      inset="0"
      justifyContent="center"
      onClick={handleClose}
      p="16px"
      position="fixed"
      zIndex="9999"
    >
      <VStack
        bg="$containerBackground"
        borderRadius="16px"
        maxH="600px"
        maxW="480px"
        onClick={(e) => e.stopPropagation()}
        overflow="hidden"
        w="min(402px, calc(100vw - 32px), calc((100dvh - 96px) * 0.75))"
      >
        <Box
          aria-label="이용자 설문조사 참여하기"
          as="a"
          aspectRatio="3 / 4"
          cursor="pointer"
          display="block"
          href={SURVEY_URL}
          overflow="hidden"
          rel="noopener noreferrer"
          target="_blank"
          w="100%"
        >
          <Image
            alt="Braillify 이용자 설문조사 안내"
            display="block"
            h="100%"
            objectFit="cover"
            src="/images/home/popup.png"
            w="100%"
          />
        </Box>
        <Flex borderTop="1px solid $border" flexShrink="0">
          <Box
            as="button"
            bg="transparent"
            border="none"
            color="$caption"
            cursor="pointer"
            flex="1"
            onClick={handleHideForToday}
            py="16px"
            type="button"
          >
            <Text typography="bodyBold">오늘 하루 안 보기</Text>
          </Box>
          <Box bg="$border" flexShrink="0" w="1px" />
          <Box
            as="button"
            bg="transparent"
            border="none"
            color="$text"
            cursor="pointer"
            flex="1"
            onClick={handleClose}
            py="16px"
            type="button"
          >
            <Text typography="bodyBold">닫기</Text>
          </Box>
        </Flex>
      </VStack>
    </Flex>
  )
}
