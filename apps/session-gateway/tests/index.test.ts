import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import type http from 'http';

let server: http.Server | undefined;
const originalEnv = { ...process.env };

describe('index bootstrap', () => {
  beforeEach(() => {
    process.env = { ...originalEnv };
  });

  afterEach(async () => {
    if (server) {
      await new Promise<void>((resolve) => server!.close(() => resolve()));
      server = undefined;
    }
    process.env = { ...originalEnv };
  });

  it('starts the gateway server on the configured port', async () => {
    process.env.PORT = '0';
    process.env.SCHEDULER_URL = 'http://scheduler.test';
    const module = await import('../src/index.js');
    server = await module.startGateway();
    const address = server.address();
    expect(typeof address === 'object' && address?.port).toBeGreaterThan(0);
  });
});
