import type { MetadataRoute } from 'next'

// Required for `output: 'export'` in Next.js — without this, Next refuses
// to pre-render the route handler at build time.
export const dynamic = 'force-static'

/**
 * Web App Manifest. Compatible with `output: 'export'`.
 * Generated at build time as /manifest.webmanifest.
 *
 * The path is wired into the document via `metadata.manifest` in
 * `app/layout.tsx`, so search engines and "Add to Home Screen"
 * surfaces discover it automatically.
 */
export default function manifest(): MetadataRoute.Manifest {
  return {
    name: 'Braillify · 2024 개정 한국 점자 변환 오픈소스 라이브러리',
    short_name: 'Braillify',
    description:
      'Braillify는 2024 개정 한국 점자 규정 기반의 오픈소스 점자 변환(점역) 라이브러리입니다. 한국어 텍스트를 실시간으로 점자로 변환하며 Node.js, Python, Rust, WebAssembly를 모두 지원합니다.',
    start_url: '/',
    scope: '/',
    display: 'standalone',
    orientation: 'portrait-primary',
    background_color: '#EFEEEB',
    theme_color: '#EFEEEB',
    lang: 'ko-KR',
    dir: 'ltr',
    categories: ['education', 'accessibility', 'developer', 'utilities'],
    icons: [
      {
        src: '/favicon.svg',
        sizes: 'any',
        type: 'image/svg+xml',
        purpose: 'any',
      },
      {
        src: '/favicon.svg',
        sizes: 'any',
        type: 'image/svg+xml',
        purpose: 'maskable',
      },
    ],
    screenshots: [
      {
        src: '/og-image.png',
        sizes: '1200x630',
        type: 'image/png',
        form_factor: 'wide',
        label: 'Braillify - 한국어 점자 변환 오픈소스 라이브러리',
      },
    ],
    shortcuts: [
      {
        name: '문서 둘러보기',
        short_name: 'Docs',
        description: 'Braillify 공식 문서로 이동',
        url: '/docs/overview',
      },
      {
        name: '설치 가이드',
        short_name: 'Install',
        description: 'Node.js, Python, Rust, .NET 설치 방법',
        url: '/docs/installation',
      },
      {
        name: '테스트 케이스',
        short_name: 'Tests',
        description: '분야별 점역 테스트 케이스 모음',
        url: '/test-case',
      },
    ],
  }
}
