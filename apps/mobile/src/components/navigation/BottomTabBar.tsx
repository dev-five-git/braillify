import { Grid, Text, VStack } from '@devup-ui/react'

import { type AppView, SIDEBAR_ITEMS } from '@/constants/navigation'

type BottomTabBarProps = {
  activeView: AppView
  onNavigate: (view: AppView) => void
}

// 좁은 화면(모바일)에서만 노출되는 하단 탭 내비게이션.
// 넓은 화면에서는 AppSidebar가 대신 노출된다.
export function BottomTabBar({ activeView, onNavigate }: BottomTabBarProps) {
  return (
    <Grid
      aria-label="주요 화면"
      as="nav"
      bg="$sidebarBackground"
      borderTop="1px solid $sidebarBorder"
      bottom="0"
      display={['grid', null, 'none']}
      gridTemplateColumns={`repeat(${SIDEBAR_ITEMS.length}, 1fr)`}
      left="0"
      pb="calc(env(safe-area-inset-bottom, 0px) + 6px)"
      position="fixed"
      pt="8px"
      right="0"
      zIndex={10}
    >
      {SIDEBAR_ITEMS.map((item) => {
        const isActive = item.id === activeView

        return (
          <VStack
            key={item.id}
            alignItems="center"
            aria-current={isActive ? 'page' : undefined}
            as="button"
            bg="transparent"
            border="none"
            color={isActive ? '$sidebarText' : '$sidebarDisabled'}
            cursor="pointer"
            gap="3px"
            onClick={() => onNavigate(item.id)}
            py="4px"
            type="button"
          >
            <Text
              aria-hidden="true"
              color="currentColor"
              fontFamily="Segoe UI Symbol, sans-serif"
              fontSize="20px"
              lineHeight="1"
            >
              {item.symbol}
            </Text>
            <Text color="currentColor" typography="sidebarCaption">
              {item.label}
            </Text>
          </VStack>
        )
      })}
    </Grid>
  )
}
