'use client'
import { Box, Flex, Text, VStack } from '@devup-ui/react'
import { useRef, useState } from 'react'

import { MathTrans } from './MathTrans'
import { Trans } from './Trans'

const TABS = [
  { key: 'korean', label: '한글' },
  { key: 'math', label: '수학' },
] as const

type DemoMode = (typeof TABS)[number]['key']

const PANEL_ID = 'demo-panel'
const tabId = (key: DemoMode) => `demo-tab-${key}`

export function DemoTabs() {
  const [mode, setMode] = useState<DemoMode>('korean')
  const tabRefs = useRef<(HTMLElement | null)[]>([])

  const handleKeyDown = (e: React.KeyboardEvent, index: number) => {
    if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft') return
    e.preventDefault()
    const dir = e.key === 'ArrowRight' ? 1 : -1
    const next = (index + dir + TABS.length) % TABS.length
    setMode(TABS[next].key)
    tabRefs.current[next]?.focus()
  }

  return (
    <VStack gap={['16px', null, null, '30px']} w="100%">
      <Flex
        aria-label="점역 데모 종류"
        gap="10px"
        justifyContent={['center', null, null, 'flex-start']}
        role="tablist"
      >
        {TABS.map(({ key, label }, index) => (
          <Text
            key={key}
            ref={(el: HTMLElement | null) => {
              tabRefs.current[index] = el
            }}
            aria-controls={PANEL_ID}
            aria-selected={mode === key}
            as="button"
            bg={mode === key ? '$text' : 'transparent'}
            border="1px solid $text"
            borderRadius="1000px"
            color={mode === key ? '$background' : '$text'}
            cursor="pointer"
            id={tabId(key)}
            onClick={() => setMode(key)}
            onKeyDown={(e) => handleKeyDown(e, index)}
            px={['20px', null, null, '28px']}
            py={['8px', null, null, '10px']}
            role="tab"
            tabIndex={mode === key ? 0 : -1}
            type="button"
            typography="button"
          >
            {label}
          </Text>
        ))}
      </Flex>
      <Box
        aria-labelledby={tabId(mode)}
        display="flex"
        flexDirection="column"
        id={PANEL_ID}
        role="tabpanel"
        w="100%"
      >
        {mode === 'korean' ? <Trans /> : <MathTrans />}
      </Box>
    </VStack>
  )
}
