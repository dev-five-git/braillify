import { expect, test } from 'bun:test'

import { applicationIssue } from './together-application'

const application = {
  kind: '학교',
  name: '점자 학교',
  status: '사용 중',
  product: '학습 앱',
  url: 'https://example.com/?a=1&b=2',
  purpose: '안내 문장을 점자로 보여 줍니다.',
  methods: ['Node.js', 'Rust'],
  since: '2026년',
  consent: true,
}

test('신청 내용을 GitHub 이슈 주소와 본문에 보존한다', () => {
  const result = applicationIssue(application)
  const url = new URL(result.url)
  expect(url.searchParams.get('title')).toContain(application.name)
  expect(url.searchParams.get('body')).toBe(result.body)
  expect(result.body).toContain(application.url)
  expect(result.body).toContain('Node.js, Rust')
  expect(result.copyRequired).toBe(false)
})

test('긴 신청 내용은 본문을 복사하도록 짧은 주소를 제공한다', () => {
  const result = applicationIssue({
    ...application,
    purpose: '가'.repeat(5000),
  })
  expect(result.copyRequired).toBe(true)
  expect(new URL(result.url).searchParams.has('body')).toBe(false)
  expect(result.body).toContain('가'.repeat(5000))
})
