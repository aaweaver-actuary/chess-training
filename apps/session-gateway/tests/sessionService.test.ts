import { describe, expect, it, vi } from 'vitest';
import { createSessionService } from '../src/sessionService.js';
import type { SessionState } from '../src/types.js';
import { createBroadcaster } from '../src/broadcaster.js';
import type { Broadcaster } from '../src/broadcaster.js';

describe('session service', () => {
  it('throws when attempting to grade without an active session', async () => {
    const schedulerClient = {
      fetchQueue: vi.fn(),
      gradeCard: vi.fn(),
    };
    const sessionStore = {
      create: vi.fn(),
      get: vi.fn().mockResolvedValue(undefined),
      update: vi.fn(),
      delete: vi.fn(),
    };
    const service = createSessionService({
      schedulerClient,
      sessionStore,
      broadcaster: createBroadcaster(),
    });

    await expect(
      service.grade({
        sessionId: 'missing',
        cardId: 'card',
        grade: 'Good',
        latencyMs: 10,
      }),
    ).rejects.toThrow('invalid-session');
  });

  it('handles zero total reviews when computing accuracy', async () => {
    const sessionState: SessionState = {
      sessionId: 's',
      userId: 'u',
      queue: [],
      currentCard: {
        card_id: 'card',
        kind: 'Opening',
        position_fen: 'fen',
        prompt: 'prompt',
      },
      stats: { reviews_today: -1, accuracy: 0, avg_latency_ms: 0 },
      totalLatency: 0,
    };
    const schedulerClient = {
      fetchQueue: vi.fn(),
      gradeCard: vi.fn().mockResolvedValue(null),
    };
    const sessionStore = {
      create: vi.fn(),
      get: vi.fn().mockResolvedValue(sessionState),
      update: vi.fn(async (_id: string, updater: (state: SessionState) => SessionState) =>
        updater(sessionState),
      ),
      delete: vi.fn(),
    };
    const broadcaster = createBroadcaster();
    const service = createSessionService({
      schedulerClient,
      sessionStore,
      broadcaster,
    });

    const result = await service.grade({
      sessionId: 's',
      cardId: 'card',
      grade: 'Again',
      latencyMs: 0,
    });
    expect(result.stats.accuracy).toBe(0);
  });

  it('starts, reads stats, and ends sessions', async () => {
    const schedulerClient = {
      fetchQueue: vi
        .fn()
        .mockResolvedValue([
          { card_id: 'card', kind: 'Opening', position_fen: 'fen', prompt: 'prompt' },
        ]),
      gradeCard: vi.fn(),
    };
    const createdStates: SessionState[] = [];
    const sessionStore = {
      create: vi.fn(async (_id: string, state: SessionState) => {
        createdStates.push(state);
      }),
      get: vi.fn().mockResolvedValue(undefined),
      update: vi.fn(),
      delete: vi.fn(),
    };
    const service = createSessionService({
      schedulerClient,
      sessionStore,
      broadcaster: createBroadcaster(),
    });

    const startResult = await service.start('user');
    expect(startResult.firstCard).toMatchObject({ card_id: 'card' });
    sessionStore.get.mockResolvedValue(createdStates[0]);
    const stats = await service.stats(startResult.sessionId);
    expect(stats).toEqual({ reviews_today: 0, accuracy: 0, avg_latency_ms: 0 });
    await service.end(startResult.sessionId);
    expect(sessionStore.delete).toHaveBeenCalledWith(startResult.sessionId);
  });

  it('grades cards, updates stats, and broadcasts results', async () => {
    const initialState: SessionState = {
      sessionId: 's',
      userId: 'u',
      queue: [],
      currentCard: {
        card_id: 'card-1',
        kind: 'Opening',
        position_fen: 'fen',
        prompt: 'prompt',
      },
      stats: { reviews_today: 1, accuracy: 0.5, avg_latency_ms: 1000 },
      totalLatency: 1000,
    };
    const nextCard = {
      card_id: 'card-2',
      kind: 'Tactic' as const,
      position_fen: 'fen2',
      prompt: 'prompt2',
    };
    const schedulerClient = {
      fetchQueue: vi.fn(),
      gradeCard: vi.fn().mockResolvedValue(nextCard),
    };
    const updateSpy = vi.fn(
      async (_id: string, updater: (state: SessionState) => SessionState) =>
        updater(initialState),
    );
    const sessionStore = {
      create: vi.fn(),
      get: vi.fn().mockResolvedValue(initialState),
      update: updateSpy,
      delete: vi.fn(),
    };
    const broadcast = vi.fn();
    const service = createSessionService({
      schedulerClient,
      sessionStore,
      broadcaster: {
        register: vi.fn(),
        unregister: vi.fn(),
        broadcast,
      } as unknown as Broadcaster,
    });

    const result = await service.grade({
      sessionId: 's',
      cardId: 'card-1',
      grade: 'Good',
      latencyMs: 500,
    });

    expect(result.nextCard).toEqual(nextCard);
    expect(result.stats).toMatchObject({ reviews_today: 2, avg_latency_ms: 750 });
    expect(result.stats.accuracy).toBeCloseTo(0.75, 2);
    expect(updateSpy).toHaveBeenCalled();
    expect(broadcast).toHaveBeenCalledWith('s', { type: 'UPDATE', card: nextCard });
    expect(broadcast).toHaveBeenCalledWith('s', {
      type: 'STATS',
      stats: expect.objectContaining({ reviews_today: 2 }),
    });
  });
});
