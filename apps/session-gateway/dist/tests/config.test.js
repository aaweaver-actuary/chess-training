import { afterEach, beforeEach, describe, expect, it } from 'vitest';
const ORIGINAL_ENV = { ...process.env };
const resetEnv = () => {
  process.env = { ...ORIGINAL_ENV };
};
describe('config', () => {
  beforeEach(() => {
    resetEnv();
  });
  afterEach(() => {
    resetEnv();
  });
  it('uses defaults when environment variables are not provided', async () => {
    delete process.env.PORT;
    delete process.env.SCHEDULER_URL;
    delete process.env.LOG_LEVEL;
    const { loadConfig } = await import('../src/config.js');
    const config = loadConfig();
    expect(config).toMatchObject({
      PORT: 3000,
      SCHEDULER_URL: 'http://localhost:4000',
      LOG_LEVEL: 'info',
    });
  });
  it('parses custom environment and trims scheduler url trailing slash', async () => {
    process.env.PORT = '4123';
    process.env.SCHEDULER_URL = 'http://example.com/api/';
    process.env.LOG_LEVEL = 'debug';
    const { loadConfig } = await import('../src/config.js');
    const config = loadConfig();
    expect(config).toMatchObject({
      PORT: 4123,
      SCHEDULER_URL: 'http://example.com/api',
      LOG_LEVEL: 'debug',
    });
  });
});
