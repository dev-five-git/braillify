'use client'

import { Box, Center, css, Flex, Text, VStack } from '@devup-ui/react'
import Link from 'next/link'
import { useEffect, useRef, useState } from 'react'

import {
  consentControl,
  Controls as Choice,
  errorMessage,
  field,
  primaryAction,
  secondaryAction,
} from '@/components/together/Controls'
import { applicationIssue } from '@/lib/together-application'
import {
  type ApplicationErrors,
  type ApplicationField,
  validateApplication,
} from '@/lib/together-validation'

const kinds = ['기업', '기관', '학교', '단체']
const requiredMark = css({ color: '$togetherError', marginLeft: '4px' })
const dateHint = css({
  color: '$togetherPlaceholder',
  typography: 'body',
  fontSize: '16px',
  lineHeight: ['normal', null, null, '1.8'],
})
const formField = css({
  display: 'flex',
  flexDirection: 'column',
  gap: ['8px', null, null, '10px'],
  color: '$title',
  typography: 'bodyBold',
})
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
  const [since, setSince] = useState('')
  const [dateFocused, setDateFocused] = useState(false)
  const [selected, setSelected] = useState<string[]>([])
  const [issue, setIssue] = useState<ReturnType<
    typeof applicationIssue
  > | null>(null)
  const [copied, setCopied] = useState(false)
  const [errors, setErrors] = useState<ApplicationErrors>({})
  const dialog = useRef<HTMLDialogElement>(null)
  const pendingFocus = useRef<ApplicationField | null>(null)
  useEffect(() => {
    const params = new URLSearchParams(window.location.search)
    const initialKind = params.get('kind')
    if (initialKind && kinds.includes(initialKind)) setKind(initialKind)
    setName((params.get('name') || '').slice(0, 20))
  }, [])
  useEffect(() => {
    if (issue) dialog.current?.showModal()
  }, [issue])
  useEffect(() => {
    if (!pendingFocus.current) return
    document.getElementById(`application-${pendingFocus.current}`)?.focus()
    pendingFocus.current = null
  }, [errors])

  function clearError(fieldName: ApplicationField) {
    setErrors((current) => {
      if (!current[fieldName]) return current
      const next = { ...current }
      delete next[fieldName]
      return next
    })
  }

  function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const form = event.currentTarget
    const data = new FormData(form)
    const dateInput = form.elements.namedItem('since') as HTMLInputElement
    if (!dateInput.validity.valid) {
      dateInput.reportValidity()
      return
    }
    const urlInput = form.elements.namedItem('url') as HTMLInputElement
    const nextErrors = validateApplication({
      name,
      product: String(data.get('product') || ''),
      purpose: String(data.get('purpose') || ''),
      methods: selected,
      consent: data.has('consent'),
      urlInvalid: urlInput.validity.typeMismatch,
    })
    const first = (
      ['name', 'product', 'url', 'purpose', 'methods', 'consent'] as const
    ).find((fieldName) => nextErrors[fieldName])
    pendingFocus.current = first || null
    setErrors(nextErrors)
    if (first) {
      return
    }
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
        <Text color="$text" typography="body">
          Braillify를 실제로 쓰고 있는 곳을 확인해 ‘함께하는 곳’에 소개합니다.
          신청은 GitHub 이슈로 받고, 매주 한 번 확인해요.
        </Text>
      </VStack>
      <Flex flexWrap="wrap" gap={['32px', null, null, '80px']}>
        <Box
          as="form"
          flex="1"
          minW="min(100%, 320px)"
          noValidate
          onSubmit={submit}
        >
          <VStack gap={['40px', null, null, '56px']}>
            <Box
              as="fieldset"
              border="none"
              display="flex"
              flexDir="column"
              gap={['24px', null, null, '32px']}
            >
              <Text
                as="legend"
                color="$title"
                mb={['12px', null, null, '16px']}
                typography="featureTitle"
              >
                함께할 곳
              </Text>
              <Box bg="$primary" h="1px" />
              <VStack gap={['8px', null, null, '10px']}>
                <Text color="$title" typography="bodyBold">
                  구분
                  <span aria-hidden="true" className={requiredMark}>
                    *
                  </span>
                </Text>
                <Flex
                  aria-label="구분"
                  aria-required="true"
                  flexWrap="wrap"
                  gap="8px"
                  role="radiogroup"
                >
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
              </VStack>
              <label className={formField}>
                <span>
                  이름
                  <span aria-hidden="true" className={requiredMark}>
                    *
                  </span>
                  <Text
                    as="span"
                    color="$text"
                    display="block"
                    typography="docsCaption"
                  >
                    목록과 인증 이미지에 이 이름으로 보여요. 20자까지.
                  </Text>
                </span>
                <input
                  aria-describedby={errors.name ? 'error-name' : undefined}
                  aria-invalid={errors.name ? true : undefined}
                  className={field}
                  data-registration-field
                  id="application-name"
                  maxLength={20}
                  onChange={(e) => {
                    setName(e.target.value)
                    if (e.target.value.trim()) clearError('name')
                  }}
                  placeholder="예: 점자 교육 연구회"
                  required
                  value={name}
                />
                {errors.name && (
                  <span className={errorMessage} id="error-name">
                    {errors.name}
                  </span>
                )}
              </label>
              <VStack gap={['8px', null, null, '10px']}>
                <Text color="$title" typography="bodyBold">
                  로고
                  <span aria-hidden="true" className={requiredMark}>
                    *
                  </span>
                </Text>
                <Flex
                  alignItems="flex-start"
                  bg="$containerBackground"
                  border="1.5px dashed $primary"
                  borderRadius="12px"
                  gap="10px"
                  justifyContent={['flex-start', null, null, 'center']}
                  px={['12px', null, null, '16px']}
                  py={['14px', null, null, '16px']}
                >
                  <Box
                    aria-hidden="true"
                    bg="$text"
                    flexShrink="0"
                    h={['25.2px', null, null, '28.8px']}
                    maskImage={[
                      'url(/images/together/notice-mobile.svg)',
                      null,
                      null,
                      'url(/images/together/notice-desktop.svg)',
                    ]}
                    maskPosition="center"
                    maskRepeat="no-repeat"
                    maskSize="contain"
                    w="20px"
                  />
                  <Text color="$text" typography="body" wordBreak="keep-all">
                    로고 파일은 다음 단계 GitHub 화면에서 올려요. PNG·JPG 1개,
                    2MB 이하.
                  </Text>
                </Flex>
              </VStack>
            </Box>
            <Box
              as="fieldset"
              border="none"
              display="flex"
              flexDir="column"
              gap={['24px', null, null, '32px']}
            >
              <Text
                as="legend"
                color="$title"
                mb={['12px', null, null, '16px']}
                typography="featureTitle"
              >
                사용 내용
              </Text>
              <Box bg="$primary" h="1px" />
              <VStack gap={['8px', null, null, '10px']}>
                <Box>
                  <Text color="$title" display="block" typography="bodyBold">
                    사용 상태
                    <span aria-hidden="true" className={requiredMark}>
                      *
                    </span>
                  </Text>
                  <Text color="$text" display="block" typography="docsCaption">
                    도입 준비 중이면 사용을 시작한 뒤 소개해 드려요.
                  </Text>
                </Box>
                <Flex
                  aria-label="사용 상태"
                  aria-required="true"
                  flexWrap="wrap"
                  gap="8px"
                  role="radiogroup"
                >
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
              </VStack>
              <label className={formField}>
                <span>
                  사용처 이름
                  <span aria-hidden="true" className={requiredMark}>
                    *
                  </span>
                  <Text
                    as="span"
                    color="$text"
                    display="block"
                    typography="docsCaption"
                  >
                    Braillify를 쓰는 서비스·제품·업무 이름
                  </Text>
                </span>
                <input
                  aria-describedby={
                    errors.product ? 'error-product' : undefined
                  }
                  aria-invalid={errors.product ? true : undefined}
                  className={field}
                  data-registration-field
                  id="application-product"
                  name="product"
                  onChange={(e) => {
                    if (e.target.value.trim()) clearError('product')
                  }}
                  placeholder="예: 민원 안내 누리집"
                  required
                />
                {errors.product && (
                  <span className={errorMessage} id="error-product">
                    {errors.product}
                  </span>
                )}
              </label>
              <label className={formField}>
                <span>
                  사용처 주소{' '}
                  <Text as="span" color="$text" typography="body">
                    (선택)
                  </Text>
                  <Text
                    as="span"
                    color="$text"
                    display="block"
                    typography="docsCaption"
                  >
                    누리집·앱 주소
                  </Text>
                </span>
                <input
                  aria-describedby={errors.url ? 'error-url' : undefined}
                  aria-invalid={errors.url ? true : undefined}
                  className={field}
                  data-registration-field
                  id="application-url"
                  name="url"
                  onChange={(e) => {
                    if (!e.target.validity.typeMismatch) clearError('url')
                  }}
                  placeholder="예: https://example.kr"
                  type="url"
                />
                {errors.url && (
                  <span className={errorMessage} id="error-url">
                    {errors.url}
                  </span>
                )}
              </label>
              <label className={formField}>
                <span>
                  어디에 어떤 용도로 쓰나요
                  <span aria-hidden="true" className={requiredMark}>
                    *
                  </span>
                </span>
                <textarea
                  aria-describedby={
                    errors.purpose ? 'error-purpose' : undefined
                  }
                  aria-invalid={errors.purpose ? true : undefined}
                  className={field}
                  data-multiline
                  data-registration-field
                  id="application-purpose"
                  name="purpose"
                  onChange={(e) => {
                    if (e.target.value.trim()) clearError('purpose')
                  }}
                  placeholder="예: 수업 자료의 문장을 점자로 바꿔 학생에게 나눠 줘요."
                  required
                  rows={5}
                />
                {errors.purpose && (
                  <span className={errorMessage} id="error-purpose">
                    {errors.purpose}
                  </span>
                )}
              </label>
              <VStack gap={['8px', null, null, '10px']}>
                <Box id="methods" tabIndex={-1}>
                  <Text color="$title" display="block" typography="bodyBold">
                    사용 방식
                    <span aria-hidden="true" className={requiredMark}>
                      *
                    </span>
                  </Text>
                  <Text color="$text" display="block" typography="docsCaption">
                    여러 개 고를 수 있어요
                  </Text>
                </Box>
                <Flex
                  aria-describedby={
                    errors.methods ? 'error-methods' : undefined
                  }
                  aria-labelledby="methods"
                  flexWrap="wrap"
                  gap="8px"
                  role="group"
                >
                  {methods.map((value, index) => (
                    <Choice
                      key={value}
                      aria-describedby={
                        errors.methods ? 'error-methods' : undefined
                      }
                      aria-invalid={errors.methods ? true : undefined}
                      checked={selected.includes(value)}
                      id={index === 0 ? 'application-methods' : undefined}
                      onChange={(e) => {
                        setSelected(
                          e.target.checked
                            ? [...selected, value]
                            : selected.filter((item) => item !== value),
                        )
                        if (e.target.checked) clearError('methods')
                      }}
                      type="checkbox"
                    >
                      {value}
                    </Choice>
                  ))}
                </Flex>
                {errors.methods && (
                  <span className={errorMessage} id="error-methods">
                    {errors.methods}
                  </span>
                )}
              </VStack>
              <label className={formField}>
                <span>
                  사용 시작 시기{' '}
                  <Text as="span" color="$text" typography="body">
                    (선택)
                  </Text>
                  <Text
                    as="span"
                    color="$text"
                    display="block"
                    typography="docsCaption"
                  >
                    연도-월-일
                  </Text>
                </span>
                <Box position="relative">
                  <input
                    className={field}
                    data-registration-field
                    name="since"
                    onBlur={() => setDateFocused(false)}
                    onChange={(event) => setSince(event.target.value)}
                    onFocus={() => setDateFocused(true)}
                    style={{
                      color: !since && !dateFocused ? 'transparent' : undefined,
                    }}
                    type="date"
                    value={since}
                  />
                  {!since && !dateFocused && (
                    <Text
                      aria-hidden="true"
                      className={dateHint}
                      left={['12px', null, null, '16px']}
                      pointerEvents="none"
                      position="absolute"
                      top="50%"
                      transform="translateY(-50%)"
                    >
                      예: 2026-09-01
                    </Text>
                  )}
                </Box>
              </label>
            </Box>
            <Box
              as="fieldset"
              border="none"
              display="flex"
              flexDir="column"
              gap={['24px', null, null, '32px']}
            >
              <Text
                as="legend"
                color="$title"
                mb={['12px', null, null, '16px']}
                typography="featureTitle"
              >
                공개 동의
              </Text>
              <Box bg="$primary" h="1px" />
              <VStack gap={['8px', null, null, '10px']}>
                <Text color="$title" typography="bodyBold">
                  공개 동의
                  <span aria-hidden="true" className={requiredMark}>
                    *
                  </span>
                </Text>
                <Flex as="label" gap="12px">
                  <input
                    aria-describedby={
                      errors.consent ? 'error-consent' : undefined
                    }
                    aria-invalid={errors.consent ? true : undefined}
                    className={consentControl}
                    id="application-consent"
                    name="consent"
                    onChange={(e) => {
                      if (e.target.checked) clearError('consent')
                    }}
                    required
                    type="checkbox"
                  />
                  <Box as="span">
                    <Text
                      as="span"
                      color="$text"
                      display="block"
                      typography="body"
                    >
                      이름과 로고를 braillify.kr ‘함께하는 곳’에 게시하는 데
                      동의합니다.
                    </Text>
                    <Text
                      as="span"
                      color="$text"
                      display="block"
                      mt="4px"
                      typography="docsCaption"
                    >
                      게시를 멈추고 싶으면 contact@devfive.kr로 알려 주세요.
                    </Text>
                  </Box>
                </Flex>
                {errors.consent && (
                  <span className={errorMessage} id="error-consent">
                    {errors.consent}
                  </span>
                )}
              </VStack>
            </Box>
            <Flex
              bg="$menuHover"
              borderRadius="12px"
              gap="10px"
              px={['16px', null, null, '20px']}
              py={['14px', null, null, '16px']}
            >
              <Box
                aria-hidden="true"
                bg="$text"
                flexShrink="0"
                h={['25.2px', null, null, '28.8px']}
                maskImage={[
                  'url(/images/together/notice-mobile.svg)',
                  null,
                  null,
                  'url(/images/together/notice-desktop.svg)',
                ]}
                maskPosition="center"
                maskRepeat="no-repeat"
                maskSize="contain"
                w="20px"
              />
              <Text
                color="$text"
                flex="1"
                typography="body"
                wordBreak="keep-all"
              >
                신청 내용과 로고는 GitHub 이슈로 공개돼요. 이메일·전화번호 같은
                개인 정보는 쓰지 마세요.
              </Text>
            </Flex>
            <Flex
              flexDir={['column-reverse', null, null, 'row']}
              gap="10px"
              justifyContent="space-between"
            >
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
          gap={['16px', null, null, '20px']}
          order={[-1, null, null, 0]}
          p={['20px', null, null, '32px']}
          position={['static', null, null, 'sticky']}
          top={['auto', null, null, '120px']}
          w={['100%', null, null, '440px']}
        >
          <Text
            color="$title"
            display={['block', null, null, 'none']}
            typography="featureTitle"
          >
            신청은 이렇게 진행돼요
          </Text>
          <Text
            color="$title"
            display={['none', null, null, 'block']}
            typography="featureCount"
          >
            신청은 이렇게 진행돼요
          </Text>
          <VStack as="ol" gap={['12px', null, null, '14px']} listStyle="none">
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
                <Text color="$text" typography="body" wordBreak="keep-all">
                  {step}
                </Text>
              </Flex>
            ))}
          </VStack>
          <Box aria-hidden="true" bg="$border" h="1px" />
          <Text color="$text" typography="docsCaption" wordBreak="keep-all">
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
          width: [
            'calc(100% - 32px)',
            null,
            null,
            'min(640px, calc(100% - 32px))',
          ],
          maxWidth: 'none',
          maxHeight: '90vh',
          overflow: 'auto',
          background: '$containerBackground',
          color: '$text',
          border: 'none',
          borderRadius: '24px',
          boxShadow: '0 12px 40px 0 #0000003D',
          padding: ['24px', null, null, '40px'],
          selectors: { '&::backdrop': { background: '#00000080' } },
        })}
        onClose={() => setIssue(null)}
        onPointerDown={(event) => {
          if (event.target !== event.currentTarget) return
          const rect = event.currentTarget.getBoundingClientRect()
          if (
            event.clientX < rect.left ||
            event.clientX > rect.right ||
            event.clientY < rect.top ||
            event.clientY > rect.bottom
          )
            event.currentTarget.close()
        }}
      >
        <VStack gap={['16px', null, null, '28px']}>
          <Flex gap="16px">
            <Text
              as="h2"
              color="$title"
              flex="1"
              id="dialog-title"
              m="0"
              typography="featureTitle"
            >
              GitHub에서 신청을 마쳐 주세요
            </Text>
            <Box
              _focusVisible={{ outline: '2px solid $link' }}
              aria-label="닫기"
              as="button"
              autoFocus
              bg="transparent"
              border="none"
              borderRadius="100px"
              cursor="pointer"
              onClick={() => dialog.current?.close()}
              p="8px"
              type="button"
            >
              <Box
                bg="$text"
                boxSize="24px"
                maskImage="url(/images/together/dialog-close.svg)"
                maskPosition="center"
                maskRepeat="no-repeat"
                maskSize="contain"
              />
            </Box>
          </Flex>
          <Text color="$text" typography="body" wordBreak="keep-all">
            {issue?.copyRequired
              ? '쓰신 내용이 길어서 이슈 본문을 미리 채우지 못했어요. 내용을 복사한 뒤 GitHub에 붙여 넣어 주세요.'
              : '신청은 Braillify GitHub 저장소의 이슈로 받아요. 쓰신 내용은 이슈 본문에 미리 채워 둘게요.'}
          </Text>
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
          <VStack
            as="ol"
            gap={['12px', null, null, '14px']}
            listStyle="none"
            m="0"
            p="0"
          >
            {[
              '[GitHub 열기]를 눌러 로그인해요.',
              '미리 채워진 본문 아래에 로고 파일(PNG·JPG, 2MB 이하)을 끌어다 놓아요.',
              '[Create]를 누르면 신청이 끝나요. 결과는 이슈에 남겨 드려요.',
            ].map((step, index) => (
              <Flex key={step} as="li" gap="12px">
                <Center
                  bg="$primary"
                  borderRadius="12px"
                  boxSize="24px"
                  flexShrink="0"
                >
                  <Text color="$base" typography="tinyBtn">
                    {index + 1}
                  </Text>
                </Center>
                <Text
                  color="$text"
                  flex="1"
                  typography="body"
                  wordBreak="keep-all"
                >
                  {step}
                </Text>
              </Flex>
            ))}
          </VStack>
          <Flex justifyContent={['stretch', null, null, 'flex-end']}>
            <Flex
              _focusVisible={{ outline: '2px solid $link' }}
              alignItems="center"
              aria-label="GitHub 열기 (새 창)"
              as="a"
              bg="$primary"
              borderRadius="1000px"
              color="$base"
              gap={['10px', null, null, '12px']}
              href={issue?.url}
              justifyContent="center"
              pl={['24px', null, null, '32px']}
              pr={['20px', null, null, '28px']}
              py={['13px', null, null, '17px']}
              rel="noopener noreferrer"
              target="_blank"
              textDecoration="none"
              w={['100%', null, null, 'auto']}
            >
              <Text
                display={['block', null, null, 'none']}
                typography="buttonLg"
              >
                GitHub 열기
              </Text>
              <Text display={['none', null, null, 'block']} typography="button">
                GitHub 열기
              </Text>
              <Box
                bg="$base"
                boxSize="20px"
                maskImage="url(/images/together/dialog-external.svg)"
                maskPosition="center"
                maskRepeat="no-repeat"
                maskSize="contain"
              />
            </Flex>
          </Flex>
          <Text color="$text" typography="docsCaption" wordBreak="keep-all">
            GitHub 계정이 없으면 신청 내용과 로고를 contact@devfive.kr로 보내
            주세요.
          </Text>
        </VStack>
      </dialog>
    </VStack>
  )
}
