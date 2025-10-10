import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { sessionGateway } from '../sessionGateway';

const fetchMock = vi.fn<typeof fetch>();

describe('sessionGateway', () => {
  beforeEach(() => {
    fetchMock.mockReset();
    globalThis.fetch = fetchMock as unknown as typeof fetch;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('starts a session with the expected payload', async () => {
    const responseBody = {
      session_id: 's1',
      queue_size: 3,
      first_card: { card_id: 'c1' },
    };
    fetchMock.mockResolvedValue(
      new Response(JSON.stringify(responseBody), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      }),
    );

    const result = await sessionGateway.startSession('user-1');

    expect(fetchMock).toHaveBeenCalledWith(
      'http://localhost:3000/api/session/start',
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({ user_id: 'user-1' }),
      }),
    );
    const firstCall = fetchMock.mock.calls[0];
    const init = firstCall[1];
    expect(init).toBeDefined();
    const headers = init?.headers;
    expect(headers).toBeInstanceOf(Headers);
    expect((headers as Headers).get('content-type')).toBe('application/json');
    expect(result).toEqual(responseBody);
  });

  it('throws when startSession fails', async () => {
    fetchMock.mockResolvedValue(new Response(null, { status: 500 }));

    await expect(sessionGateway.startSession('user-2')).rejects.toThrow(
      '/api/session/start failed: 500',
    );
  });

  it('submits a grade and returns the next card', async () => {
    const responseBody = { next_card: { card_id: 'c2' } };
    fetchMock.mockResolvedValue(
      new Response(JSON.stringify(responseBody), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      }),
    );

    const result = await sessionGateway.grade('c1', 'Good', 4500);

    expect(fetchMock).toHaveBeenCalledWith(
      'http://localhost:3000/api/session/grade',
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({
          card_id: 'c1',
          grade: 'Good',
          latency_ms: 4500,
        }),
      }),
    );
    const gradeCall = fetchMock.mock.calls[0];
    const [, gradeInit] = gradeCall;
    expect(gradeInit).toBeDefined();
    const gradeHeaders = gradeInit?.headers;
    expect(gradeHeaders).toBeInstanceOf(Headers);
    expect((gradeHeaders as Headers).get('content-type')).toBe('application/json');
    expect(result).toEqual(responseBody);
  });

  it('throws when grade fails', async () => {
    fetchMock.mockResolvedValue(new Response(null, { status: 400 }));

    await expect(sessionGateway.grade('c9', 'Again', 1000)).rejects.toThrow(
      '/api/session/grade failed: 400',
    );
  });

  it('fetches stats', async () => {
    const responseBody = { reviews_today: 3 };
    fetchMock.mockResolvedValue(
      new Response(JSON.stringify(responseBody), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      }),
    );

    const result = await sessionGateway.stats();

    expect(fetchMock).toHaveBeenCalledWith(
      'http://localhost:3000/api/session/stats',
      expect.objectContaining({ method: 'GET' }),
    );
    expect(result).toEqual(responseBody);
  });

  it('throws when stats fails', async () => {
    fetchMock.mockResolvedValue(new Response(null, { status: 404 }));

    await expect(sessionGateway.stats()).rejects.toThrow('/api/session/stats failed: 404');
  });
});
