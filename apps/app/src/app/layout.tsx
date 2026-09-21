import { globalCss } from '@devup-ui/react'
import type { Metadata, Viewport } from 'next'
import localFont from 'next/font/local'

const spoqaHanSansNeo = localFont({
  display: 'swap',
  src: './fonts/SpoqaHanSansNeo-Regular.woff2',
  variable: '--font-spoqa-han-sans-neo',
  weight: '400',
})

globalCss({
  'html, body': {
    minHeight: '100%',
  },
  body: {
    background: '$background',
    color: '$text',
    fontFamily: 'Segoe UI, Malgun Gothic, sans-serif',
    overflowX: 'hidden',
    WebkitTapHighlightColor: 'transparent',
  },
  '*': {
    boxSizing: 'border-box',
    margin: 0,
    padding: 0,
  },
  'button, textarea': {
    font: 'inherit',
  },
  'button:focus-visible, textarea:focus-visible, [tabindex]:focus-visible': {
    outlineWidth: '3px',
    outlineStyle: 'solid',
    outlineColor: '$focus',
    outlineOffset: '2px',
  },
  '::selection': {
    background: '$focus',
    color: '$base',
  },
})

export const metadata: Metadata = {
  applicationName: 'Braillify',
  description: '2024 개정 한국 점자 규정 기반의 오프라인 점역기',
  title: 'Braillify',
}

export const viewport: Viewport = {
  colorScheme: 'light',
  themeColor: '#F3F2EF',
  width: 'device-width',
  initialScale: 1,
  viewportFit: 'cover',
}

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html className={spoqaHanSansNeo.variable} lang="ko">
      <head>
        <link href="/favicon.svg" rel="icon" />
      </head>
      <body>{children}</body>
    </html>
  )
}
