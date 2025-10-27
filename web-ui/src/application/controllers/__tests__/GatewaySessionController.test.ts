import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { CardSummary, SessionStats, StartSessionResponse } from '../../../types/gateway';
import { GatewaySessionController, type SessionGateway } from '../GatewaySessionController';

describe('GatewaySessionController', () => {
  const gateway = {
    startSession: vi.fn<Parameters<SessionGateway['startSession']>, ReturnType<SessionGateway['startSession']>>(),
    grade: vi.fn<Parameters<SessionGateway['grade']>, ReturnType<SessionGateway['grade']>>(),
    stats: vi.fn<Parameters<SessionGateway['stats']>, ReturnType<SessionGateway['stats']>>(),
  };

  const stubCard: CardSummary = {
    card_id: 'card-1',
    kind: 'Opening',
    position_fen: 'start',
    prompt: 'Play the best move',
  };

  const stubStats: SessionStats = {
    reviews_today: 0,
    accuracy: 0.8,
    avg_latency_ms: 1500,
    due_count: 10,
    completed_count: 2,
  };

  let controller: GatewaySessionController;

  beforeEach(() => {
    gateway.startSession.mockReset();
    gateway.grade.mockReset();
    gateway.stats.mockReset();

    controller = new GatewaySessionController(gateway as unknown as SessionGateway);
  });

  it('starts a demo session, loads stats, and emits a snapshot', async () => {
    const response: StartSessionResponse = {
      session_id: 'session-1',
      queue_size: 3,
      first_card: stubCard,
    };
    gateway.startSession.mockResolvedValue(response);
    gateway.stats.mockResolvedValue(stubStats);

    const listener = vi.fn();
    controller.subscribe(listener);

    await controller.startDemo();

    expect(gateway.startSession).toHaveBeenCalledWith('demo-user');
    expect(gateway.stats).toHaveBeenCalledWith('session-1');

    const snapshot = controller.getSnapshot();
    expect(snapshot.sessionId).toBe('session-1');
    expect(snapshot.currentCard).toEqual(stubCard);
    expect(snapshot.stats).toEqual(stubStats);
    expect(listener).toHaveBeenCalledWith(expect.objectContaining({ currentCard: stubCard }));
  });

  it('submits a grade and advances to the next card', async () => {
    gateway.startSession.mockResolvedValue({
      session_id: 'session-1',
      queue_size: 3,
      first_card: stubCard,
    });
    gateway.stats.mockResolvedValue(stubStats);

    await controller.startDemo();

    const nextCard: CardSummary = { ...stubCard, card_id: 'card-2' };
    gateway.grade.mockResolvedValue({ next_card: nextCard, stats: { ...stubStats, completed_count: 3 } });
    gateway.stats.mockResolvedValue(stubStats);

    await controller.submitGrade('Good', 2500);

    expect(gateway.grade).toHaveBeenCalledWith('session-1', 'card-1', 'Good', 2500);
    expect(controller.getSnapshot().currentCard).toEqual(nextCard);
    expect(controller.getSnapshot().stats?.completed_count).toBe(3);
    expect(gateway.stats).toHaveBeenCalledTimes(1);
  });

  it('ignores grade submissions when no current card is active', async () => {
    await controller.submitGrade('Good', 1200);

    expect(gateway.grade).not.toHaveBeenCalled();
  });

  it('allows subscribers to unsubscribe', async () => {
    const listener = vi.fn();
    const unsubscribe = controller.subscribe(listener);

    await controller.startDemo();
    listener.mockClear();

    unsubscribe();
    controller.reset();

    expect(listener).not.toHaveBeenCalled();
  });
});
