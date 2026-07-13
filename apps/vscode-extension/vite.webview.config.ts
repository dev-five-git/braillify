import { DevupUI } from '@devup-ui/vite-plugin'
import react from '@vitejs/plugin-react'
import path from 'path'
import { defineConfig } from 'vite'
import topLevelAwait from 'vite-plugin-top-level-await'
import wasm from 'vite-plugin-wasm'

export default defineConfig({
  plugins: [react(), wasm(), topLevelAwait(), DevupUI()],
  build: {
    outDir: 'out/webview',
    emptyOutDir: true,
    // VS Code 웹뷰는 리소스를 재작성된 URI로만 서빙하므로 루트경로(/index_bg.wasm)
    // fetch가 불가능하다. WASM을 base64 data URL로 인라인하여 fetch/경로 문제를
    // 원천 차단한다. (그 외 에셋은 기본 규칙 유지)
    assetsInlineLimit: (filePath: string) =>
      filePath.endsWith('.wasm') ? true : undefined,
    rollupOptions: {
      input: {
        index: path.resolve(__dirname, 'webview/main.tsx'),
      },
      output: {
        entryFileNames: '[name].js',
        assetFileNames: '[name].[ext]',
        chunkFileNames: '[name].js',
      },
    },
    target: 'esnext',
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './webview'),
    },
  },
  ssr: {
    noExternal: ['@braillify/shared'],
  },
})
