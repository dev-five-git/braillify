import { Flex, Text, VStack } from '@devup-ui/react'

import { BrailleComposerControls } from './BrailleComposerControls'

// 서버 컴포넌트: 정적 골격(제목·설명·레이아웃)만 렌더한다.
// 점 선택·미리보기·버튼 등 상호작용은 BrailleComposerControls(client island)가 맡는다.
export function BrailleComposer() {
  return (
    <Flex
      alignItems={['flex-start', null, 'center']}
      bg="$background"
      borderTop="1px solid $border"
      flexWrap="wrap"
      gap={['12px', null, '20px']}
      justifyContent="space-between"
      px="24px"
      py="18px"
    >
      <VStack
        flex={['1 1 0', null, '0 0 auto']}
        gap="6px"
        minW={['0', null, '148px']}
      >
        <Text typography="inputTitle">6점 입력기</Text>
        <Text color="$caption" typography="sidebarCaption">
          점을 고른 뒤 셀을 추가하세요.
        </Text>
      </VStack>

      <BrailleComposerControls />
    </Flex>
  )
}
