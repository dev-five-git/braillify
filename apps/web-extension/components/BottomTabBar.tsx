import { Box, Flex, Text } from '@devup-ui/react'

export type TabKey = 'translator' | 'editor' | 'history'

const TABS: Array<{ key: TabKey; label: string; icon: string }> = [
  { key: 'translator', label: '점역기', icon: '⠿' },
  { key: 'editor', label: '편집기', icon: '⠶' },
  { key: 'history', label: '히스토리', icon: '⠒' },
]

type Props = {
  active: TabKey
  onChange: (key: TabKey) => void
}

export function BottomTabBar({ active, onChange }: Props) {
  return (
    <Box
      as="nav"
      bg="$tabBar"
      bottom={0}
      display="grid"
      gridTemplateColumns="repeat(3, 1fr)"
      pb="calc(env(safe-area-inset-bottom, 0px) + 8px)"
      position="sticky"
      pt="10px"
    >
      {TABS.map((tab) => {
        const isActive = tab.key === active
        return (
          <Flex
            key={tab.key}
            alignItems="center"
            as="button"
            bg="transparent"
            border={0}
            color={isActive ? '#FFFFFF' : '#9A9A9A'}
            flexDirection="column"
            gap="4px"
            justifyContent="center"
            onClick={() => onChange(tab.key)}
            p="6px"
            style={{ cursor: 'pointer' }}
            type="button"
          >
            {/* 활성 dot 인디케이터 */}
            <Box
              alignItems="center"
              display="flex"
              height="28px"
              justifyContent="center"
              position="relative"
            >
              {isActive && (
                <Box
                  left="50%"
                  position="absolute"
                  style={{
                    transform: 'translateX(-50%) translateX(-1px)',
                    width: '4px',
                    height: '4px',
                    borderRadius: '50%',
                    backgroundColor: '#16887f',
                  }}
                  top="0"
                />
              )}
              <Text fontSize="20px" lineHeight="1" mt="6px">
                {tab.icon}
              </Text>
            </Box>
            <Text fontSize="12px" fontWeight={isActive ? 600 : 400}>
              {tab.label}
            </Text>
          </Flex>
        )
      })}
    </Box>
  )
}
