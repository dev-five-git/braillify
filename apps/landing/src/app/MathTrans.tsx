import { VStack } from '@devup-ui/react'

import { DemoHeading } from './DemoHeading'
import { MathTransPanel } from './MathTransPanel'

export function MathTrans() {
  return (
    <VStack gap={['16px', null, null, '30px']}>
      <DemoHeading>
        수식도 점자가 됩니다. 수식 키보드로 입력해 수학 점역을 체험해보세요!
      </DemoHeading>
      <MathTransPanel />
    </VStack>
  )
}
