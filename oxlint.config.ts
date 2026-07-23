import config from 'eslint-plugin-devup/oxlint-config'

export default {
  ...config,
  ignorePatterns: [
    ...(config.ignorePatterns ?? []),
    'scripts/**',
    'test_cases/**',
  ],
}
