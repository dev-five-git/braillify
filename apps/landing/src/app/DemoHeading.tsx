import { Box, Flex, Text } from '@devup-ui/react'

/** 데모 섹션 상단의 손가락 아이콘 + 안내 문구. 한글·수학 데모가 공유한다. */
export function DemoHeading({ children }: { children: string }) {
  return (
    <Flex
      alignItems="flex-start"
      gap={['10px', null, null, '20px']}
      justifyContent={['center', null, null, 'flex-start']}
    >
      <Box
        aria-hidden="true"
        bg="$text"
        flexShrink={0}
        h={['20px', null, null, '32px']}
        maskImage="url(/images/home/finger-point.svg)"
        maskPosition="center"
        maskRepeat="no-repeat"
        maskSize="contain"
        // 아이콘 원본 비율(28x32)상 h=20px 의 이론 폭은 17.5px 이다.
        // maskSize=contain 이라 박스를 18px 로 잡아도 아이콘은 늘어나지 않는다.
        w={['18px', null, null, '28px']}
      />
      <Text color="$text" pos="relative" top="-2px" typography="mainTextSm">
        {children}
      </Text>
    </Flex>
  )
}
