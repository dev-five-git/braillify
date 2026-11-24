'use client'

import { Box, Center, css, Flex, Text, VStack } from '@devup-ui/react'
import Link from 'next/link'
import { useEffect, useRef, useState } from 'react'

import {
  consentControl,
  Controls as Choice,
  field,
  fieldLabel,
  primaryAction,
  secondaryAction,
} from '@/components/together/Controls'
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
            <Box
              as="fieldset"
              border="none"
              display="flex"
              flexDir="column"
              gap="16px"
            >
              <Text as="legend" color="$title" typography="featureTitle">
                함께할 곳
              </Text>
              <Box bg="$primary" h="1px" />
              <Text color="$title" typography="bodyBold">
                구분 *
              </Text>
              <Flex flexWrap="wrap" gap="8px">
                {kinds.map((value) => (
                  <Choice
                    key={value}
                    checked={kind === value}
                    name="kind"
                    onChange={() => setKind(value)}
                    type="radio"
                  >
                    {value}
                  </Choice>
                ))}
              </Flex>
              <p>
                <label className={fieldLabel}>
                  이름 *{' '}
                  <Text as="span" color="$text" typography="docsCaption">
                    목록과 인증 이미지에 이 이름으로 보여요. 20자까지.
                  </Text>
                  <input
                    className={field}
                    maxLength={20}
                    onChange={(e) => setName(e.target.value)}
                    required
                    value={name}
                  />
                </label>
              </p>
              <Box
                bg="$containerBackground"
                border="1.5px dashed $primary"
                borderRadius="12px"
                p="16px"
              >
                로고 파일은 다음 단계 GitHub 화면에서 올려요. PNG·JPG 1개, 2MB
                이하.
              </Box>
            </Box>
            <Box
              as="fieldset"
              border="none"
              display="flex"
              flexDir="column"
              gap="16px"
            >
              <Text as="legend" color="$title" typography="featureTitle">
                사용 내용
              </Text>
              <Box bg="$primary" h="1px" />
              <Text color="$title" typography="bodyBold">
                사용 상태 *
              </Text>
              <Text color="$text" typography="docsCaption">
                도입 준비 중이면 사용을 시작한 뒤 소개해 드려요.
              </Text>
              <Flex flexWrap="wrap" gap="8px">
                {['사용 중', '도입 준비 중'].map((value, index) => (
                  <Choice
                    key={value}
                    defaultChecked={index === 0}
                    name="status"
                    type="radio"
                    value={value}
                  >
                    {value}
                  </Choice>
                ))}
              </Flex>
              <p>
                <label className={fieldLabel}>
                  사용처 이름 *{' '}
                  <Text as="span" color="$text" typography="docsCaption">
                    Braillify를 쓰는 서비스·제품·업무 이름
                  </Text>
                  <input className={field} name="product" required />
                </label>
              </p>
              <p>
                <label className={fieldLabel}>
                  사용처 주소 (선택){' '}
                  <input className={field} name="url" type="url" />
                </label>
              </p>
              <p>
                <label className={fieldLabel}>
                  어디에 어떤 용도로 쓰나요 *{' '}
                  <textarea
                    className={field}
                    data-multiline
                    name="purpose"
                    required
                    rows={5}
                  />
                </label>
              </p>
              <p id="methods" tabIndex={-1}>
                사용 방식 * (여러 개 고를 수 있어요)
              </p>
              <Flex flexWrap="wrap" gap="8px">
                {methods.map((value) => (
                  <Choice
                    key={value}
                    checked={selected.includes(value)}
                    onChange={(e) =>
                      setSelected(
                        e.target.checked
                          ? [...selected, value]
                          : selected.filter((item) => item !== value),
                      )
                    }
                    type="checkbox"
                  >
                    {value}
                  </Choice>
                ))}
              </Flex>
              <p>
                <label className={fieldLabel}>
                  사용 시작 시기 (선택){' '}
                  <input className={field} name="since" type="date" />
                </label>
              </p>
            </Box>
            <Box
              as="fieldset"
              border="none"
              display="flex"
              flexDir="column"
              gap="16px"
            >
              <Text as="legend" color="$title" typography="featureTitle">
                공개 동의
              </Text>
              <Box bg="$primary" h="1px" />
              <Flex as="label" gap="12px">
                <input
                  className={consentControl}
                  name="consent"
                  required
                  type="checkbox"
                />
                <Text as="span" color="$text">
                  이름과 로고를 braillify.kr ‘함께하는 곳’에 게시하는 데
                  동의합니다.
                </Text>
              </Flex>
              <p>게시를 멈추고 싶으면 contact@devfive.kr로 알려 주세요.</p>
            </Box>
            {error && <p role="alert">{error}</p>}
            <Text color="$text">
              신청 내용과 로고는 GitHub 이슈로 공개돼요. 이메일·전화번호 같은
              개인 정보는 쓰지 마세요.
            </Text>
            <Flex justifyContent="space-between">
              <Link className={secondaryAction} href="/together">
                이전으로
              </Link>
              <button className={primaryAction} type="submit">
                신청하기
              </button>
            </Flex>
          </VStack>
        </Box>
        <VStack
          alignSelf="flex-start"
          bg="$containerBackground"
          borderRadius="20px"
          gap="20px"
          order={[-1, null, null, 0]}
          p="32px"
          position={['static', null, null, 'sticky']}
          top={['auto', null, null, '120px']}
          w={['100%', null, null, '440px']}
        >
          <Text color="$title" typography="featureTitle">
            신청은 이렇게 진행돼요
          </Text>
          <VStack as="ol" gap="14px" listStyle="none">
            {[
              '이 페이지에서 신청 내용을 써요',
              'GitHub 이슈에 로고를 올려 제출해요',
              '매주 한 번 실제 사용을 확인해요',
              '승인되면 목록에 소개하고 이슈로 알려 드려요',
            ].map((step, index) => (
              <Flex key={step} as="li" gap="12px">
                <Center
                  aria-hidden="true"
                  bg="$primary"
                  borderRadius="12px"
                  boxSize="24px"
                  flexShrink="0"
                >
                  <Text color="$base" typography="tinyBtn">
                    {index + 1}
                  </Text>
                </Center>
                <Text color="$text">{step}</Text>
              </Flex>
            ))}
          </VStack>
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
          borderRadius: '24px',
          boxShadow: '0 12px 40px 0 #0000003D',
          padding: ['24px', null, null, '32px'],
        })}
        onClose={() => setIssue(null)}
      >
        <VStack gap="16px">
          <Text
            as="h2"
            color="$title"
            id="dialog-title"
            typography="featureTitle"
          >
            GitHub에서 신청을 마쳐 주세요
          </Text>
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
              <textarea
                aria-label="신청 내용"
                className={field}
                readOnly
                rows={5}
                value={issue.body}
              />
              <button
                className={secondaryAction}
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
          <p>
            1. GitHub에서 로그인해요.
            <br />
            2. 로고 파일(PNG·JPG, 2MB 이하)을 첨부해요.
            <br />
            3. Create를 누르면 신청이 끝나요.
          </p>
          <a
            className={primaryAction}
            href={issue?.url}
            rel="noopener noreferrer"
            target="_blank"
          >
            GitHub 열기 (새 창)
          </a>
          <button
            className={secondaryAction}
            onClick={() => dialog.current?.close()}
            type="button"
          >
            닫기
          </button>
        </VStack>
      </dialog>
    </VStack>
  )
}
