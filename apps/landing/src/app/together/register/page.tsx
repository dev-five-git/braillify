import type { Metadata } from 'next'

import Register from './Register'

export const metadata: Metadata = { title: '함께하는 곳 신청' }

export default function RegisterPage() {
  return <Register />
}
