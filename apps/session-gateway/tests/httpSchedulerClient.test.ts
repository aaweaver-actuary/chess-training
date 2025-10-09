import { describe, expect, it, vi } from 'vitest';
import { createHttpSchedulerClient } from '../src/clients/httpSchedulerClient.js';

const jsonResponse = (body: unknown, init: ResponseInit = {}) => {
  const headers = new Headers(init.headers);
  if (!headers.has('content-type')) {
    headers.set('content-type', 'application/json');
  }
  return new Response(JSON.stringify(body), {
    status: init.status ?? 200,
    ...init,
    headers,
  });
};

const createClient = (fetchImpl?: ReturnType<typeof vi.fn>) =>
  createHttpSchedulerClient({ baseUrl: 'http://scheduler.test', fetchImpl });

const sampleCard = {
  card_id: 'card-1',
  kind: 'Opening' as const,
  position_fen: 'start',
  prompt: 'Go',
};

describe('http scheduler client', () => {
  it('fetches queues and next cards from scheduler', async () => {
    const fetchMock = vi.fn(async () => jsonResponse({ queue: [sampleCard] }));
    const client = createClient(fetchMock);
    const queue = await client.fetchQueue('user-1');
    expect(queue).toEqual([sampleCard]);
    expect(fetchMock).toHaveBeenCalledWith(
      'http://scheduler.test/queue',
      expect.anything(),
    );

    fetchMock.mockResolvedValueOnce(jsonResponse({ next_card: sampleCard }));
    const next = await client.gradeCard({
      sessionId: 'session-1',
      cardId: 'card-1',
      grade: 'Good',
      latencyMs: 1000,
    });
    expect(next).toEqual(sampleCard);
    fetchMock.mockResolvedValueOnce(new Response('{}', { status: 200 }));
    const emptyQueue = await client.fetchQueue('user-1');
    expect(emptyQueue).toEqual([]);
    fetchMock.mockResolvedValueOnce(new Response('', { status: 200 }));
    const emptyQueueNoBody = await client.fetchQueue('user-1');
    expect(emptyQueueNoBody).toEqual([]);
    fetchMock.mockResolvedValueOnce(new Response('{}', { status: 200 }));
    const missingNext = await client.gradeCard({
      sessionId: 'session-1',
      cardId: 'card-1',
      grade: 'Good',
      latencyMs: 1000,
    });
    expect(missingNext).toBeNull();

    fetchMock.mockResolvedValueOnce(new Response('', { status: 200 }));
    const emptyResponse = await client.gradeCard({
      sessionId: 'session-1',
      cardId: 'card-2',
      grade: 'Hard',
      latencyMs: 1500,
    });
    expect(emptyResponse).toBeNull();
  });

  it('throws when scheduler responds with non-ok status', async () => {
    const fetchMock = vi.fn(async () => new Response('fail', { status: 500 }));
    const client = createClient(fetchMock);
    await expect(client.fetchQueue('user-1')).rejects.toThrow('scheduler-error');
    fetchMock.mockResolvedValueOnce(new Response('nope', { status: 500 }));
    await expect(
      client.gradeCard({ sessionId: 's', cardId: 'c', grade: 'Again', latencyMs: 10 }),
    ).rejects.toThrow('scheduler-error');
  });

  it('falls back to global fetch implementation when none is provided', async () => {
    const originalFetch = global.fetch;
    const fetchSpy = vi.fn(
      async () => new Response(JSON.stringify({ queue: [] }), { status: 200 }),
    );
    global.fetch = fetchSpy as typeof fetch;
    const client = createClient();
    await client.fetchQueue('user-2');
    expect(fetchSpy).toHaveBeenCalled();
    global.fetch = originalFetch;
  });

  it('treats missing queue payloads as empty results', async () => {
    const fetchMock = vi.fn();
    fetchMock.mockResolvedValueOnce(new Response(JSON.stringify({}), { status: 200 }));
    const client = createHttpSchedulerClient({
      baseUrl: 'http://scheduler.test',
      fetchImpl: fetchMock,
    });
  it('returns an empty queue when the scheduler omits the queue field', async () => {
    const fetchMock = vi.fn();
    // Response missing the 'queue' field
    fetchMock.mockResolvedValueOnce(jsonResponse({ not_queue: [] }, { headers: {} }));
    const client = createClient(fetchMock);
    await expect(client.fetchQueue('user-3')).resolves.toEqual([]);
  });
});
