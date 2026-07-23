import { Box, globalCss, ThemeScript } from '@devup-ui/react'
import type { Metadata, Viewport } from 'next'
import localFont from 'next/font/local'

import Footer from '@/components/Footer'
import Header from '@/components/Header'

const middleKoreanFont = localFont({
  src: './fonts/NanumBarunGothic-YetHangul.woff2',
  variable: '--font-middle-korean',
  display: 'swap',
})

const SITE_URL = 'https://braillify.kr'
const SITE_NAME = 'Braillify'
const SITE_TITLE = 'Braillify · 2024 개정 한국 점자 변환 오픈소스 라이브러리'
const SITE_DESCRIPTION =
  'Braillify는 2024 개정 한국 점자 규정 기반의 오픈소스 점자 변환 라이브러리입니다. 한국어 텍스트를 실시간으로 점자(점역)로 변환하며 Node.js, Python, Rust, WebAssembly를 모두 지원합니다.'

export const metadata: Metadata = {
  metadataBase: new URL(SITE_URL),
  title: {
    default: SITE_TITLE,
    template: '%s | Braillify · 한국어 점자 변환 라이브러리',
  },
  description: SITE_DESCRIPTION,
  applicationName: SITE_NAME,
  generator: 'Next.js',
  referrer: 'origin-when-cross-origin',
  authors: [
    { name: 'Devfive', url: 'https://www.themoredream.com' },
    { name: 'dev-five-git', url: 'https://github.com/dev-five-git' },
  ],
  creator: 'Devfive (데브파이브)',
  publisher: 'Devfive (데브파이브)',
  category: 'technology',
  classification:
    'Open Source Software, Accessibility, Korean Braille Translator',
  keywords: [
    // Primary brand
    'Braillify',
    'braillify',
    '브레일리파이',
    '브레일파이',
    '브레이리파이',
    // Common typos / misspellings (people who can't spell it)
    'brailify',
    'braillfy',
    'brailfy',
    'brailliffy',
    'braillify 점자',
    'braillify 점역',
    // Core domain - Korean (변환/번역/점역 all common search terms)
    '한국어 점자 변환',
    '한국어 점자 번역',
    '한국 점자 변환',
    '한국 점자 번역',
    '한글 점자 변환',
    '한글 점자 번역',
    '한글 점자 변환기',
    '한글 점자 번역기',
    '한국어 점자 변환기',
    '한글 점역',
    '한국어 점역',
    '한글 점역 라이브러리',
    '한국어 점역 라이브러리',
    '한글 점역 라이브러리 추천',
    // Spacing variants (Korean users often vary spacing)
    '점자변환',
    '점자 변환',
    '점자번역',
    '점자 번역',
    '점자변환기',
    '점자 변환기',
    '점자번역기',
    '점자 번역기',
    '점역기',
    '점역 기',
    '점역프로그램',
    '점역 프로그램',
    '점자통역',
    '점자 통역',
    '점자통역기',
    '점자 통역기',
    // Library / dev terms
    '점자 라이브러리',
    '점자 SDK',
    '점자 API',
    '점자 오픈소스',
    '점자 변환 라이브러리',
    '점자 번역 라이브러리',
    '점자 통역 라이브러리',
    '점자 라이브러리 추천',
    '오픈소스 점역기',
    // Standards & specs
    '2024 개정 한국 점자 규정',
    '2024 한국 점자',
    '한국 점자 규정',
    '점자 표기법',
    '점자 표기',
    '점자 표준',
    // Platforms
    'Node.js 점자',
    'Python 점자',
    'Rust 점자',
    'WebAssembly 점자',
    'wasm 점자',
    // Use cases / audience
    '시각장애인 점자',
    '점자 출력',
    '점자 인쇄',
    '점자 학습',
    '실시간 점역',
    '실시간 점자 변환',
    // Bidirectional
    '한글 to 점자',
    '점자 ↔ 한글',
    '점자 ↔ 영어',
    // Competitors / alternatives (people searching for alternatives)
    '점사랑 대체',
    '하상브레일 대체',
    '오픈소스 점역기 추천',
    // English
    'Korean braille translator',
    'Korean braille library',
    'braille translator',
    'braille korean',
    'braille english',
    'Hangul to braille',
    'Korean braille converter',
    'open source braille translator',
    'braille SDK',
    'braille API',
    'braille WASM',
    'braille npm package',
    'braille python package',
    'braille rust crate',
  ],
  alternates: {
    canonical: '/',
    languages: {
      ko: '/',
      'ko-KR': '/',
      'x-default': '/',
    },
  },
  manifest: '/manifest.webmanifest',
  icons: {
    icon: [{ url: '/favicon.svg', type: 'image/svg+xml' }],
    shortcut: '/favicon.svg',
    apple: '/favicon.svg',
  },
  robots: {
    index: true,
    follow: true,
    nocache: false,
    googleBot: {
      index: true,
      follow: true,
      'max-image-preview': 'large',
      'max-snippet': -1,
      'max-video-preview': -1,
    },
  },
  openGraph: {
    type: 'website',
    locale: 'ko_KR',
    alternateLocale: ['en_US'],
    url: SITE_URL,
    siteName: SITE_NAME,
    title: SITE_TITLE,
    description: SITE_DESCRIPTION,
    images: [
      {
        url: '/og-image.png',
        width: 1200,
        height: 630,
        alt: 'Braillify - 한국어 점자 변환 오픈소스 라이브러리',
        type: 'image/png',
      },
    ],
  },
  twitter: {
    card: 'summary_large_image',
    title: SITE_TITLE,
    description: SITE_DESCRIPTION,
    images: ['/og-image.png'],
    creator: '@devfive',
    site: '@devfive',
  },
  formatDetection: {
    email: false,
    address: false,
    telephone: false,
  },
  // Search engine site verification. Inject codes via env vars at build time:
  //   NEXT_PUBLIC_GOOGLE_SITE_VERIFICATION   — Google Search Console
  //   NEXT_PUBLIC_NAVER_SITE_VERIFICATION    — Naver Search Advisor
  //   NEXT_PUBLIC_YANDEX_SITE_VERIFICATION   — Yandex Webmaster
  //   NEXT_PUBLIC_YAHOO_SITE_VERIFICATION    — Yahoo Site Explorer
  //   NEXT_PUBLIC_BING_SITE_VERIFICATION     — Bing Webmaster (msvalidate.01)
  // Empty / undefined vars are simply omitted, so no stray <meta> is emitted.
  verification: {
    ...(process.env.NEXT_PUBLIC_GOOGLE_SITE_VERIFICATION && {
      google: process.env.NEXT_PUBLIC_GOOGLE_SITE_VERIFICATION,
    }),
    ...(process.env.NEXT_PUBLIC_YANDEX_SITE_VERIFICATION && {
      yandex: process.env.NEXT_PUBLIC_YANDEX_SITE_VERIFICATION,
    }),
    ...(process.env.NEXT_PUBLIC_YAHOO_SITE_VERIFICATION && {
      yahoo: process.env.NEXT_PUBLIC_YAHOO_SITE_VERIFICATION,
    }),
    other: {
      ...(process.env.NEXT_PUBLIC_NAVER_SITE_VERIFICATION && {
        'naver-site-verification':
          process.env.NEXT_PUBLIC_NAVER_SITE_VERIFICATION,
      }),
      ...(process.env.NEXT_PUBLIC_BING_SITE_VERIFICATION && {
        'msvalidate.01': process.env.NEXT_PUBLIC_BING_SITE_VERIFICATION,
      }),
    },
  },
  other: {
    // Korean / global SNS share enhancements
    'og:locale:alternate': 'en_US',
  },
}

export const viewport: Viewport = {
  width: 'device-width',
  initialScale: 1,
  maximumScale: 5,
  themeColor: [
    { media: '(prefers-color-scheme: light)', color: '#EFEEEB' },
    { media: '(prefers-color-scheme: dark)', color: '#000000' },
  ],
  colorScheme: 'light dark',
}

// Site-wide JSON-LD: Organization + WebSite + SoftwareApplication
const jsonLd = [
  {
    '@context': 'https://schema.org',
    '@type': 'Organization',
    '@id': `${SITE_URL}/#organization`,
    name: 'Devfive',
    alternateName: ['데브파이브', 'Dev Five', 'dev-five-git'],
    url: 'https://www.themoredream.com',
    logo: {
      '@type': 'ImageObject',
      url: `${SITE_URL}/favicon.svg`,
    },
    sameAs: [
      'https://github.com/dev-five-git',
      'https://www.npmjs.com/package/braillify',
      'https://pypi.org/project/braillify/',
      'https://crates.io/crates/braillify',
      'https://discord.gg/8zjcGc7cWh',
    ],
  },
  {
    '@context': 'https://schema.org',
    '@type': 'WebSite',
    '@id': `${SITE_URL}/#website`,
    url: SITE_URL,
    name: SITE_NAME,
    description: SITE_DESCRIPTION,
    inLanguage: 'ko-KR',
    publisher: { '@id': `${SITE_URL}/#organization` },
    potentialAction: {
      '@type': 'SearchAction',
      target: {
        '@type': 'EntryPoint',
        urlTemplate: `${SITE_URL}/test-case?q={search_term_string}`,
      },
      'query-input': 'required name=search_term_string',
    },
  },
  {
    '@context': 'https://schema.org',
    '@type': 'SoftwareApplication',
    '@id': `${SITE_URL}/#software`,
    name: 'Braillify',
    alternateName: [
      '브레일리파이',
      '한국어 점자 변환 라이브러리',
      '한글 점역 라이브러리',
      'Korean Braille Translator',
    ],
    applicationCategory: 'DeveloperApplication',
    applicationSubCategory: 'Accessibility Library',
    operatingSystem: 'Cross-platform (Node.js, Python, Rust, WebAssembly)',
    description: SITE_DESCRIPTION,
    url: SITE_URL,
    downloadUrl: 'https://www.npmjs.com/package/braillify',
    softwareHelp: `${SITE_URL}/docs/overview`,
    inLanguage: 'ko-KR',
    license: 'https://www.apache.org/licenses/LICENSE-2.0',
    isAccessibleForFree: true,
    offers: {
      '@type': 'Offer',
      price: '0',
      priceCurrency: 'USD',
    },
    author: { '@id': `${SITE_URL}/#organization` },
    publisher: { '@id': `${SITE_URL}/#organization` },
    keywords:
      '한국어 점자 변환, 한글 점역, 점자 라이브러리, 2024 개정 한국 점자 규정, 오픈소스 점역기, Korean braille translator, braille SDK',
  },
]

globalCss({
  imports: ['https://spoqa.github.io/spoqa-han-sans/css/SpoqaHanSansNeo.css'],
  'html, body': {
    maxWidth: '100vw',
    overflowX: 'hidden',
  },
  body: {
    maxHeight: '100vh',
    background: '#373634',
    // WebkitFontSmoothing: 'antialiased',
    // MozOsxFontSmoothing: 'grayscale',
    fontFamily: 'Spoqa Han Sans Neo, Arial, Helvetica, sans-serif',
    wordBreak: 'keep-all',
  },
  '*': {
    boxSizing: 'border-box',
    padding: 0,
    margin: 0,
  },
  a: {
    color: '$link',
    textDecoration: 'none',
  },
  '::placeholder': {
    fontFamily: 'Spoqa Han Sans Neo, Arial, Helvetica, sans-serif',
  },
})

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="ko" suppressHydrationWarning>
      <head>
        <script>{`
(function(w,d,s,l,i){w[l]=w[l]||[];w[l].push({'gtm.start':
new Date().getTime(),event:'gtm.js'});var f=d.getElementsByTagName(s)[0],
j=d.createElement(s),dl=l!='dataLayer'?'&l='+l:'';j.async=true;j.src=
'https://www.googletagmanager.com/gtm.js?id='+i+dl;f.parentNode.insertBefore(j,f);
})(window,document,'script','dataLayer','GTM-KHQZ6Z4V')`}</script>
        <ThemeScript auto />
        <link href="/favicon.svg" rel="shortcut icon" />
        {/*
          JSON-LD structured data for rich results.
          Embedding via <script type="application/ld+json"> is the
          schema.org-recommended approach and is safe inside <head>.
        */}
        <script
          dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
          type="application/ld+json"
        />
      </head>
      <body className={middleKoreanFont.variable}>
        <noscript>
          <iframe
            height="0"
            src="https://www.googletagmanager.com/ns.html?id=GTM-KHQZ6Z4V"
            style={{ display: 'none', visibility: 'hidden' }}
            title="Google Tag Manager"
            width="0"
          />
        </noscript>
        {/* Skip-to-content link for keyboard users — visually hidden until focused. */}
        <Box
          _focus={{
            clip: 'auto',
            clipPath: 'none',
            height: 'auto',
            left: '8px',
            top: '8px',
            width: 'auto',
            margin: '0',
            overflow: 'visible',
            padding: '8px 12px',
            bg: '$background',
            color: '$text',
            borderRadius: '8px',
            zIndex: '9999',
          }}
          as="a"
          clip="rect(0 0 0 0)"
          clipPath="inset(50%)"
          height="1px"
          href="#main-content"
          left="-9999px"
          margin="-1px"
          overflow="hidden"
          padding="0"
          position="absolute"
          whiteSpace="nowrap"
          width="1px"
        >
          본문으로 건너뛰기
        </Box>
        <Box bg="$background" pos="relative">
          <Header />
          <Box as="main" id="main-content">
            {children}
          </Box>
        </Box>
        <Footer />
      </body>
    </html>
  )
}
