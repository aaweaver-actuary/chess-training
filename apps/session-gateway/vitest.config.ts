import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    coverage: {
      reporter: ['text', 'lcov'],
      provider: 'v8',
      exclude: ['src/index.ts', 'src/types.ts', 'eslint.config.js', 'vitest.config.ts'],
      thresholds: {
        lines: 99,
        functions: 94,
        statements: 99,
        branches: 97,
      },
    },
  },
});
