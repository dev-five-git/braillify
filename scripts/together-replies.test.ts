import { expect, test } from 'bun:test'

import { approvalMessage, publishedPartner, sendApprovalReplies } from './together-replies'

const partner = { issue: 123, kind: '학교' as const, name: '점자 학교', logo: '/images/together/asset-123.png', approvedAt: '2026-11-20' }

test('승인 답글에는 고유 카드와 인증 이미지 및 두 배지 링크가 있다', () => {
  const message = approvalMessage(partner)
  expect(message).toContain('https://braillify.kr/together#org-123')
  expect(message).toContain('https://braillify.kr/together/cert/org-123.png')
  expect(message).toContain('https://braillify.kr/together/badge/org-123-light.png')
  expect(message).toContain('https://braillify.kr/together/badge/org-123-dark.png')
  expect(message).toContain('<!-- braillify-together-approval:123 -->')
})

test('배포된 공개 기록이 승인 기록과 정확히 일치할 때만 통과한다', () => {
  expect(publishedPartner([partner], partner)).toBe(true)
  expect(publishedPartner([{ ...partner, name: '다른 곳' }], partner)).toBe(false)
  expect(publishedPartner([], partner)).toBe(false)
})

test('배포 이미지가 없으면 이슈에 답글을 보내지 않는다', async () => {
  const requests: string[] = []
  const request = async (input: string | URL | Request) => {
    const url = String(input)
    requests.push(url)
    return url.endsWith('/together.json')
      ? Response.json([partner])
      : new Response('missing', { status: 404 })
  }
  await expect(sendApprovalReplies([partner], 'token', request as typeof fetch)).rejects.toThrow()
  expect(requests.some((url) => url.startsWith('https://api.github.com'))).toBe(false)
})

test('배포 확인 후에만 답글을 달고 기존 봇 답글은 반복하지 않는다', async () => {
  let sent = 0
  let existing = false
  const request = async (input: string | URL | Request, init?: RequestInit) => {
    const url = String(input)
    if (url.endsWith('/together.json')) return Response.json([partner])
    if (url.startsWith('https://braillify.kr/')) {
      expect(init?.headers).toBeUndefined()
      return new Response(new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]), { headers: { 'content-type': 'image/png' } })
    }
    if (init?.method === 'POST') { sent++; existing = true; return Response.json({}) }
    return Response.json(existing ? [{ user: { login: 'github-actions[bot]' }, body: '<!-- braillify-together-approval:123 -->' }] : [])
  }
  await sendApprovalReplies([partner], 'token', request as typeof fetch)
  await sendApprovalReplies([partner], 'token', request as typeof fetch)
  expect(sent).toBe(1)
})
