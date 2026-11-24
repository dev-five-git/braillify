export interface TogetherApplication {
  kind: string
  name: string
  status: string
  product: string
  url: string
  purpose: string
  methods: string[]
  since: string
  consent: boolean
}

export function applicationIssue(application: TogetherApplication) {
  const body = [
    '## 함께할 곳',
    `구분: ${application.kind}`,
    `이름: ${application.name}`,
    '',
    '## 사용 내용',
    `사용 상태: ${application.status}`,
    `사용처 이름: ${application.product}`,
    `사용처 주소: ${application.url || '없음'}`,
    `용도: ${application.purpose}`,
    `사용 방식: ${application.methods.join(', ')}`,
    `사용 시작 시기: ${application.since || '없음'}`,
    '',
    `공개 동의: ${application.consent ? '동의' : '동의하지 않음'}`,
    '',
    '## 로고',
    '여기에 밝은 바탕용 PNG 또는 JPG 로고 1개(2MB 이하)를 첨부해 주세요.',
  ].join('\n')

  const url = new URL('https://github.com/dev-five-git/braillify/issues/new')
  url.searchParams.set('title', `[함께하는 곳] ${application.name}`)
  url.searchParams.set('body', body)
  // shortcut: very long URLs require copying the body, until GitHub accepts larger issue URLs reliably.
  const copyRequired = url.toString().length > 1800
  if (copyRequired) url.searchParams.delete('body')
  return { body, url: url.toString(), copyRequired }
}
