import { fileURLToPath } from 'node:url'
import { mergeConfig, defineConfig, configDefaults } from 'vitest/config'
import viteConfig from './vite.config'

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      environment: 'jsdom',
      exclude: [...configDefaults.exclude, 'e2e/**'],
      root: fileURLToPath(new URL('./', import.meta.url)),
      coverage: {
        provider: 'v8',
        include: ['src/**/*.{ts,vue}'],
        reporter: ['text', 'html', 'lcov'],
        reportsDirectory: 'coverage',
      },
      reporters: [
        'default',
        ['junit', { suiteName: 'wallet_admin_portal tests', outputFile: 'test-results/junit.xml' }],
        ['allure-vitest/reporter', { resultsDir: 'allure-results' }],
      ],
    },
  }),
)
