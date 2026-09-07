'use client'

import { Input } from '@devup-ui/react'
import { ComponentProps } from 'react'

import { useTestCase } from './TestCaseProvider'

export function FailedOnlyInput(
  props: Omit<
    ComponentProps<typeof Input<'input'>>,
    'as' | 'checked' | 'onChange' | 'defaultChecked'
  >,
) {
  const { options, onChangeOptions } = useTestCase()
  return (
    <Input
      checked={options.failedOnly}
      onChange={(e) =>
        onChangeOptions({ ...options, failedOnly: e.target.checked })
      }
      {...props}
    />
  )
}
