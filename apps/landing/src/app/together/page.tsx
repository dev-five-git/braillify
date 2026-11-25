import type { Metadata } from 'next'

import Together from './Together'

export const metadata: Metadata = { title: '함께하는 곳' }

export default function TogetherPage() {
  return <Together />
}
