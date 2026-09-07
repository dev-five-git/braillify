'use client'

import { VStack } from '@devup-ui/react'
import { ComponentProps } from 'react'

import { useTestCase } from './TestCaseProvider'

export function TestCaseRuleContainer({
  exception,
  children,
  ...props
}: {
  exception: boolean
} & Omit<ComponentProps<typeof VStack<'div'>>, 'as'>) {
  const { options } = useTestCase()
  const isList = options.type === 'list'
  return (
    <VStack
      flex="1"
      gap={['20px', null, null, isList ? '30px' : '40px']}
      pb={[isList ? '30px' : '40px', null, null, '40px']}
      pt={exception ? null : [isList ? '30px' : '40px', null, null, '40px']}
      px={['16px', null, null, '60px']}
      styleOrder={1}
      {...props}
    >
      {children}
    </VStack>
  )
}
