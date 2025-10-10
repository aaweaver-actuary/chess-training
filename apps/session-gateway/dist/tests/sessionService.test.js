import { describe, expect, it, vi } from 'vitest';
import { createSessionService } from '../src/sessionService.js';
import { createBroadcaster } from '../src/broadcaster.js';
const createSchedulerMock = (overrides = {}) => ({
    fetchQueue: vi.fn(),
    gradeCard: vi.fn(),
    ...overrides,
});
const createSessionStoreMock = (overrides = {}) => ({
    create: vi.fn(),
    get: vi.fn(),
    update: vi.fn(),
    delete: vi.fn(),
    ...overrides,
});
const createSessionState = (overrides = {}) => ({
    sessionId: 's',
    userId: 'u',
    queue: [],
    currentCard: {
        card_id: 'card',
        kind: 'Opening',
        position_fen: 'fen',
        prompt: 'prompt',
    },
    stats: { reviews_today: 0, accuracy: 0, avg_latency_ms: 0 },
    totalLatency: 0,
    ...overrides,
});
const createBroadcasterMock = () => ({
    register: vi.fn(),
    unregister: vi.fn(),
    broadcast: vi.fn(),
});
describe('session service', () => {
    it('throws when attempting to grade without an active session', async () => {
        const schedulerClient = createSchedulerMock();
        const sessionStore = createSessionStoreMock({
            get: vi.fn().mockResolvedValue(undefined),
        });
        const service = createSessionService({
            schedulerClient,
            sessionStore,
            broadcaster: createBroadcaster(),
        });
        await expect(service.grade({
            sessionId: 'missing',
            cardId: 'card',
            grade: 'Good',
            latencyMs: 10,
        })).rejects.toThrow('invalid-session');
    });
    it('handles zero total reviews when computing accuracy', async () => {
        const sessionState = createSessionState({
            stats: { reviews_today: -1, accuracy: 0, avg_latency_ms: 0 },
        });
        const schedulerClient = createSchedulerMock({
            gradeCard: vi.fn().mockResolvedValue(null),
        });
        const sessionStore = createSessionStoreMock({
            get: vi.fn().mockResolvedValue(sessionState),
            update: vi.fn(async (_id, updater) => updater(sessionState)),
        });
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
        const schedulerClient = createSchedulerMock({
            fetchQueue: vi
                .fn()
                .mockResolvedValue([
                { card_id: 'card', kind: 'Opening', position_fen: 'fen', prompt: 'prompt' },
            ]),
        });
        const createdStates = [];
        const sessionStore = createSessionStoreMock({
            create: vi.fn(async (_id, state) => {
                createdStates.push(state);
            }),
            get: vi.fn().mockResolvedValue(undefined),
        });
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
        const initialState = createSessionState({
            currentCard: {
                card_id: 'card-1',
                kind: 'Opening',
                position_fen: 'fen',
                prompt: 'prompt',
            },
            stats: { reviews_today: 1, accuracy: 0.5, avg_latency_ms: 1000 },
            totalLatency: 1000,
        });
        const nextCard = {
            card_id: 'card-2',
            kind: 'Tactic',
            position_fen: 'fen2',
            prompt: 'prompt2',
        };
        const schedulerClient = createSchedulerMock({
            gradeCard: vi.fn().mockResolvedValue(nextCard),
        });
        const updateSpy = vi.fn(async (_id, updater) => updater(initialState));
        const sessionStore = createSessionStoreMock({
            get: vi.fn().mockResolvedValue(initialState),
            update: updateSpy,
        });
        const broadcaster = createBroadcasterMock();
        const service = createSessionService({
            schedulerClient,
            sessionStore,
            broadcaster: broadcaster,
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
        expect(broadcaster.broadcast).toHaveBeenCalledWith('s', expect.objectContaining({
            type: 'UPDATE',
            card: nextCard,
            stats: expect.objectContaining({ reviews_today: 2, avg_latency_ms: 750 }),
        }));
    });
});
