import { DevupUI } from '@devup-ui/vite-plugin'
import { defineConfig } from 'wxt'

// WXT/Vite 8 빌드 시 codeSplitting=false 환경에서 @devup-ui가 강제 주입하는 manualChunks 속성 오염 차단 헬퍼
const fixDevupUiPlugin = () => ({
  name: 'fix-devup-ui',
  config(config: any) {
    if (config.build?.rollupOptions?.output) {
      if (Array.isArray(config.build.rollupOptions.output)) {
        config.build.rollupOptions.output.forEach((opt: any) => {
          delete opt.manualChunks
        })
      } else {
        delete config.build.rollupOptions.output.manualChunks
      }
    }
  },
})

// See https://wxt.dev/api/config.html
export default defineConfig({
  modules: ['@wxt-dev/module-react'],
  imports: false,
  vite: () => ({
    plugins: [DevupUI({ extractCss: true }), fixDevupUiPlugin()],
    ssr: {
      noExternal: ['@braillify/shared'],
    },
    build: {
      rollupOptions: {
        external: ['wxt/utils/storage', 'wxt/storage'],
      },
    },
    optimizeDeps: {
      exclude: ['@braillify/shared'],
    },
  }),
})
