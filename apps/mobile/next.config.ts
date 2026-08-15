import { DevupUI } from '@devup-ui/next-plugin'
import type { NextConfig } from 'next'

const nextConfig: NextConfig = {
  // Tauri는 정적 export 결과물(out/)을 번들링한다.
  // SSR / Server Actions / 동적 route handler는 사용하지 않는다.
  output: 'export',

  // 정적 export에서는 Next 이미지 최적화 서버를 쓸 수 없다.
  images: {
    unoptimized: true,
  },

  // react-compiler로 클라이언트 런타임 비용을 추가로 줄인다.
  reactCompiler: true,

  webpack(config, { isServer, dev }) {
    // braillify는 wasm-pack `--target bundler` 산출물(ESM WebAssembly)이다.
    // Webpack 5는 기본적으로 WebAssembly를 비활성화하므로 직접 켜준다.
    config.experiments = { ...config.experiments, asyncWebAssembly: true }

    // 정적 export 프리렌더(server 번들)에서 .wasm 경로가 어긋나
    // "Error occurred prerendering page" 가 나는 것을 방지한다.
    config.output.webassemblyModuleFilename =
      isServer && !dev
        ? '../static/wasm/[modulehash].wasm'
        : 'static/wasm/[modulehash].wasm'

    return config
  },
}

export default DevupUI(nextConfig, {})
