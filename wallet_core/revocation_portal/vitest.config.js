import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['assets/__tests__/**/*.test.js'],
  },
});
