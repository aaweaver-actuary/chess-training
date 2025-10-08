import { describe, expect, it } from 'vitest';
import { createInMemorySessionStore } from '../src/stores/inMemoryStore.js';
import type { SessionState } from '../src/types.js';

const createState = (overrides: Partial<SessionState> = {}): SessionState => ({
  sessionId: 'session-1',
  userId: 'user-1',
  queue: [],
  currentCard: null,
  stats: { reviews_today: 0, accuracy: 0, avg_latency_ms: 0 },
  totalLatency: 0,
  ...overrides,
});

describe('in-memory session store', () => {
  it('persists, updates, and deletes session state', async () => {
    const store = createInMemorySessionStore<SessionState>();
    await store.create('session-1', createState());
    const existing = await store.get('session-1');
    expect(existing?.userId).toBe('user-1');

    const updated = await store.update('session-1', (state) => ({
      ...state,
      stats: { reviews_today: 2, accuracy: 0.5, avg_latency_ms: 1000 },
    }));
    expect(updated.stats.reviews_today).toBe(2);

    await store.delete('session-1');
    const afterDelete = await store.get('session-1');
    expect(afterDelete).toBeUndefined();
  });

  it('throws when attempting to update a missing session', async () => {
    const store = createInMemorySessionStore<SessionState>();
    await expect(store.update('missing', (state) => ({ ...state }))).rejects.toThrow(
      'session-missing',
    );
  });
});
