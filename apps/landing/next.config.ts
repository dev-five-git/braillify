import { DevupUI } from '@devup-ui/next-plugin'
import createMDX from '@next/mdx'

const withMDX = createMDX({})

export default withMDX(
  DevupUI(
    {
      webpack(config, { isServer, dev }) {
        // Use the client static directory in the server bundle and prod mode
        // Fixes `Error occurred prerendering page "/"`
        config.output.webassemblyModuleFilename =
          isServer && !dev
            ? '../static/wasm/[modulehash].wasm'
            : 'static/wasm/[modulehash].wasm'

        // Since Webpack 5 doesn't enable WebAssembly by default, we should do it manually
        config.experiments = { ...config.experiments, asyncWebAssembly: true }

        return config
      },

      pageExtensions: ['js', 'jsx', 'md', 'mdx', 'ts', 'tsx'],
      output: 'export',
    },
    {},
  ),
)
