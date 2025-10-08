import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../../clients/sessionGateway', () => ({
  sessionGateway: {
    startSession: vi.fn(),
    grade: vi.fn(),
    stats: vi.fn(),
  },
}));

import type { CardSummary, SessionStats, StartSessionResponse } from '../../types/gateway';
import { sessionGateway } from '../../clients/sessionGateway';
import { sessionStore } from '../sessionStore';

const gateway = sessionGateway as unknown as {
  startSession: ReturnType<typeof vi.fn>;
  grade: ReturnType<typeof vi.fn>;
  stats: ReturnType<typeof vi.fn>;
};

const stubCard: CardSummary = {
  card_id: 'c1',
  kind: 'Opening',
  position_fen: 'start',
  prompt: 'Play the move',
};

const stubStats: SessionStats = {
  reviews_today: 1,
  accuracy: 0.5,
  avg_latency_ms: 1500,
  due_count: 10,
  completed_count: 2,
};

beforeEach(() => {
  gateway.startSession.mockReset();
  gateway.grade.mockReset();
  gateway.stats.mockReset();
  sessionStore.setState({ sessionId: undefined, currentCard: undefined, queue: [], stats: undefined });
});

describe('sessionStore', () => {
  it('starts a session and loads stats', async () => {
    const response: StartSessionResponse = { session_id: 's1', queue_size: 3, first_card: stubCard };
    gateway.startSession.mockResolvedValue(response);
    gateway.stats.mockResolvedValue(stubStats);

    await sessionStore.getState().start('user-1');

    expect(gateway.startSession).toHaveBeenCalledWith('user-1');
    expect(sessionStore.getState().sessionId).toBe('s1');
    expect(sessionStore.getState().currentCard).toEqual(stubCard);
    expect(sessionStore.getState().stats).toEqual(stubStats);
  });

  it('submits a grade, advances to the next card, and refreshes stats', async () => {
    const refreshedStats: SessionStats = { ...stubStats, completed_count: 3 };
    gateway.grade.mockResolvedValue({ next_card: { ...stubCard, card_id: 'c2' } });
    gateway.stats.mockResolvedValue(refreshedStats);
    sessionStore.setState({ sessionId: 's1', currentCard: stubCard, stats: stubStats });

    await sessionStore.getState().submitGrade('Good', 3200);

    expect(gateway.grade).toHaveBeenCalledWith('c1', 'Good', 3200);
    expect(sessionStore.getState().currentCard?.card_id).toBe('c2');
    expect(sessionStore.getState().stats).toEqual(refreshedStats);
  });

  it('ignores grade submissions when no card is active', async () => {
    await sessionStore.getState().submitGrade('Good', 1500);

    expect(gateway.grade).not.toHaveBeenCalled();
  });

  it('allows subscribers to unsubscribe from updates', () => {
    const listener = vi.fn();
    const unsubscribe = sessionStore.subscribe(listener);

    sessionStore.setState({ stats: stubStats });
    expect(listener).toHaveBeenCalledWith(expect.objectContaining({ stats: stubStats }));

    listener.mockClear();
    unsubscribe();
    sessionStore.setState({ stats: { ...stubStats, completed_count: 5 } });

    expect(listener).not.toHaveBeenCalled();
  });
});
