'use client'

import { Box, css, Flex, Text, VStack } from '@devup-ui/react'
import Link from 'next/link'
import { useEffect, useRef, useState } from 'react'

import { applicationIssue } from '@/lib/together-application'

const kinds = ['기업', '기관', '학교', '단체']
const methods = [
  'Node.js',
  'Python',
  'Rust',
  '.NET',
  'C',
  'Go',
  'JVM',
  'Ruby',
  'HTTP API',
  '기타',
]

export default function Register() {
  const [kind, setKind] = useState('기업')
  const [name, setName] = useState('')
  const [selected, setSelected] = useState<string[]>([])
  const [issue, setIssue] = useState<ReturnType<
    typeof applicationIssue
  > | null>(null)
  const [copied, setCopied] = useState(false)
  const [error, setError] = useState('')
  const dialog = useRef<HTMLDialogElement>(null)
  useEffect(() => {
    const params = new URLSearchParams(window.location.search)
    const initialKind = params.get('kind')
    if (initialKind && kinds.includes(initialKind)) setKind(initialKind)
    setName((params.get('name') || '').slice(0, 20))
  }, [])
  useEffect(() => {
    if (issue) dialog.current?.showModal()
  }, [issue])

  function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const data = new FormData(event.currentTarget)
    if (
      !name.trim() ||
      !String(data.get('product')).trim() ||
      !String(data.get('purpose')).trim()
    ) {
      setError('필수 항목을 입력해 주세요.')
      return
    }
    if (!selected.length) {
      setError('사용 방식을 하나 이상 선택해 주세요.')
      document.getElementById('methods')?.focus()
      return
    }
    setError('')
    setIssue(
      applicationIssue({
        kind,
        name,
        status: String(data.get('status')),
        product: String(data.get('product')),
        url: String(data.get('url')),
        purpose: String(data.get('purpose')),
        methods: selected,
        since: String(data.get('since')),
        consent: data.has('consent'),
      }),
    )
  }
  return (
    <VStack
      gap="40px"
      maxW="1520px"
      mx="auto"
      px={['16px', null, '30px', '60px']}
      py="40px"
      w="100%"
    >
      <VStack gap="20px">
        <Text as="h1" color="$title" m="0" typography="docsTitle">
          함께하는 곳 신청
        </Text>
        <Text color="$text">
          Braillify를 실제로 쓰고 있는 곳을 확인해 ‘함께하는 곳’에 소개합니다.
          신청은 GitHub 이슈로 받고, 매주 한 번 확인해요.
        </Text>
      </VStack>
      <Flex flexWrap="wrap" gap="32px">
        <Box as="form" flex="1" minW="min(100%, 320px)" onSubmit={submit}>
          <VStack gap="32px">
            <fieldset>
              <legend>함께할 곳</legend>
              <p>구분 *</p>
              {kinds.map((value) => (
                <label key={value}>
                  <input
                    checked={kind === value}
                    name="kind"
                    onChange={() => setKind(value)}
                    type="radio"
                  />
                  {value}{' '}
                </label>
              ))}
              <p>
                <label>
                  이름 *{' '}
                  <input
                    maxLength={20}
                    onChange={(e) => setName(e.target.value)}
                    required
                    value={name}
                  />
                </label>
              </p>
              <p>
                로고 파일은 다음 단계 GitHub 화면에서 올려요. PNG·JPG 1개, 2MB
                이하.
              </p>
            </fieldset>
            <fieldset>
              <legend>사용 내용</legend>
              <p>사용 상태 *</p>
              {['사용 중', '도입 준비 중'].map((value, index) => (
                <label key={value}>
                  <input
                    defaultChecked={index === 0}
                    name="status"
                    type="radio"
                    value={value}
                  />
                  {value}{' '}
                </label>
              ))}
              <p>
                <label>
                  사용처 이름 * <input name="product" required />
                </label>
              </p>
              <p>
                <label>
                  사용처 주소 (선택) <input name="url" type="url" />
                </label>
              </p>
              <p>
                <label>
                  어디에 어떤 용도로 쓰나요 *{' '}
                  <textarea name="purpose" required />
                </label>
              </p>
              <p id="methods" tabIndex={-1}>
                사용 방식 * (여러 개 고를 수 있어요)
              </p>
              {methods.map((value) => (
                <label key={value}>
                  <input
                    checked={selected.includes(value)}
                    onChange={(e) =>
                      setSelected(
                        e.target.checked
                          ? [...selected, value]
                          : selected.filter((item) => item !== value),
                      )
                    }
                    type="checkbox"
                  />
                  {value}{' '}
                </label>
              ))}
              <p>
                <label>
                  사용 시작 시기 (선택) <input name="since" type="date" />
                </label>
              </p>
            </fieldset>
            <fieldset>
              <legend>공개 동의</legend>
              <label>
                <input name="consent" required type="checkbox" />
                이름과 로고를 braillify.kr ‘함께하는 곳’에 게시하는 데
                동의합니다.
              </label>
              <p>게시를 멈추고 싶으면 contact@devfive.kr로 알려 주세요.</p>
            </fieldset>
            {error && <p role="alert">{error}</p>}
            <Text color="$text">
              신청 내용과 로고는 GitHub 이슈로 공개돼요. 이메일·전화번호 같은
              개인 정보는 쓰지 마세요.
            </Text>
            <Flex justifyContent="space-between">
              <Link href="/together">이전으로</Link>
              <button type="submit">신청하기</button>
            </Flex>
          </VStack>
        </Box>
        <VStack
          bg="$containerBackground"
          borderRadius="20px"
          gap="20px"
          p="32px"
          w={['100%', null, null, '440px']}
        >
          <Text color="$title" typography="featureTitle">
            신청은 이렇게 진행돼요
          </Text>
          <Text color="$text">
            1. 이 페이지에서 신청 내용을 써요
            <br />
            2. GitHub 이슈에 로고를 올려 제출해요
            <br />
            3. 매주 한 번 실제 사용을 확인해요
            <br />
            4. 승인되면 목록에 소개하고 이슈로 알려 드려요
          </Text>
          <Text color="$text">
            GitHub 계정이 없으면 신청 내용과 로고를 contact@devfive.kr로 보내
            주세요.
          </Text>
        </VStack>
      </Flex>
      <dialog
        ref={dialog}
        aria-labelledby="dialog-title"
        className={css({
          position: 'fixed',
          inset: '0',
          margin: 'auto',
          width: 'min(90vw, 520px)',
          maxHeight: '90vh',
          overflow: 'auto',
          background: '$containerBackground',
          color: '$text',
          border: '1px solid $border',
          borderRadius: '16px',
          padding: '32px',
        })}
        onClose={() => setIssue(null)}
      >
        <h2 id="dialog-title">GitHub에서 신청하기</h2>
        <p>
          {issue?.copyRequired
            ? '쓰신 내용이 길어서 이슈 본문을 미리 채우지 못했어요. 내용을 복사한 뒤 GitHub에 붙여 넣어 주세요.'
            : '신청은 Braillify GitHub 저장소의 이슈로 받아요. 쓰신 내용은 이슈 본문에 미리 채워 둘게요.'}
        </p>
        <p>
          GitHub 계정이 없으면 신청 내용과 로고를 contact@devfive.kr로 보내
          주세요.
        </p>
        {issue?.copyRequired && (
          <>
            <textarea aria-label="신청 내용" readOnly value={issue.body} />
            <button
              onClick={() => {
                navigator.clipboard
                  .writeText(issue.body)
                  .then(() => {
                    setCopied(true)
                    setTimeout(() => setCopied(false), 2000)
                  })
                  .catch(() => setCopied(false))
              }}
              type="button"
            >
              {copied ? '복사됨' : '내용 복사하기'}
            </button>
          </>
        )}
        <p>GitHub에서 로고를 첨부한 뒤 이슈를 제출해 주세요.</p>
        <a href={issue?.url} rel="noopener noreferrer" target="_blank">
          GitHub 열기 (새 창)
        </a>
        <button onClick={() => dialog.current?.close()} type="button">
          닫기
        </button>
      </dialog>
    </VStack>
  )
}
