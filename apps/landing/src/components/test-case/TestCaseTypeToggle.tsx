'use client'

import { Toggle } from '@devup-ui/components'
import { ComponentProps } from 'react'

import { useTestCase } from './TestCaseProvider'

export function TestCaseTypeToggle(props: ComponentProps<typeof Toggle>) {
  const { options, onChangeOptions } = useTestCase()
  return (
    <Toggle
      onChange={(value) =>
        onChangeOptions({ ...options, type: value ? 'table' : 'list' })
      }
      style={{
        backgroundColor: 'var(--toggleBg)',
      }}
      value={options.type === 'table'}
      {...props}
    />
  )
}
