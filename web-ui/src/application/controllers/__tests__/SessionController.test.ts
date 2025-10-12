import { beforeEach, describe, expect, it, vi } from 'vitest';

import type {
  CardSummary,
  ReviewGrade,
  SessionStats,
  StartSessionResponse,
} from '../../../types/gateway.js';
import {
  createSessionController,
  type SessionSnapshot,
} from '../SessionController.js';

describe('createSessionController', () => {
  const sampleCard: CardSummary = {
    card_id: 'card-1',
    kind: 'Opening',
    position_fen: 'startpos',
    prompt: 'Play e4',
    expected_moves_uci: ['e2e4'],
  };
  const followUpCard: CardSummary = {
    card_id: 'card-2',
    kind: 'Tactic',
    position_fen: 'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1',
    prompt: 'Respond to e4',
    expected_moves_uci: ['c7c5'],
  };
  const sampleStats: SessionStats = {
    reviews_today: 3,
    accuracy: 0.75,
    avg_latency_ms: 1234,
    due_count: 12,
    completed_count: 8,
  };

  const gateway = {
    startSession: vi.fn<[], Promise<StartSessionResponse>>(),
    grade: vi.fn<
      [string, ReviewGrade, number],
      Promise<{ next_card?: CardSummary }>
    >(),
    stats: vi.fn<[], Promise<SessionStats>>(),
  };

  const buildController = () =>
    createSessionController({
      gateway,
      defaultUserId: 'user-123',
      demoSession: {
        sessionId: 'demo-session',
        cards: [sampleCard, followUpCard],
        stats: sampleStats,
      },
    });

  beforeEach(() => {
    vi.resetAllMocks();
  });

  it('starts idle with an empty queue', () => {
    const controller = buildController();
    expect(controller.getSnapshot()).toEqual<SessionSnapshot>({
      status: 'idle',
      queueSize: 0,
    });
  });

  it('loads a live session from the gateway', async () => {
    const controller = buildController();
    gateway.startSession.mockResolvedValue({
      session_id: 'session-99',
      queue_size: 4,
      first_card: sampleCard,
    });
    gateway.stats.mockResolvedValue(sampleStats);

    const updates: SessionSnapshot[] = [];
    controller.subscribe((snapshot) => {
      updates.push(snapshot);
    });

    await controller.start();

    expect(gateway.startSession).toHaveBeenCalledWith('user-123');
    expect(updates.at(0)?.status).toBe('loading');
    expect(updates.at(-1)).toMatchObject({
      status: 'active',
      sessionId: 'session-99',
      currentCard: sampleCard,
      queueSize: 4,
      stats: sampleStats,
      error: undefined,
    });
  });

  it('submits a grade against the active card and refreshes stats', async () => {
    const controller = buildController();
    gateway.startSession.mockResolvedValue({
      session_id: 'session-99',
      queue_size: 2,
      first_card: sampleCard,
    });
    gateway.stats.mockResolvedValue(sampleStats);
    gateway.grade.mockResolvedValue({ next_card: followUpCard });

    await controller.start();
    await controller.submitGrade('Good', 987);

    expect(gateway.grade).toHaveBeenCalledWith('card-1', 'Good', 987);
    expect(gateway.stats).toHaveBeenCalledTimes(2);
    expect(controller.getSnapshot()).toMatchObject({
      status: 'active',
      currentCard: followUpCard,
      queueSize: 1,
      lastGrade: 'Good',
      stats: sampleStats,
    });
  });

  it('supports demo sessions without talking to the gateway', async () => {
    const controller = buildController();

    await controller.startDemo();

    expect(gateway.startSession).not.toHaveBeenCalled();
    expect(controller.getSnapshot()).toMatchObject({
      status: 'active',
      sessionId: 'demo-session',
      currentCard: sampleCard,
      queueSize: 2,
    });

    await controller.submitGrade('Hard', 2500);

    expect(gateway.grade).not.toHaveBeenCalled();
    expect(controller.getSnapshot()).toMatchObject({
      status: 'active',
      currentCard: followUpCard,
      queueSize: 1,
      lastGrade: 'Hard',
    });
  });

  it('resets to an idle snapshot', async () => {
    const controller = buildController();
    gateway.startSession.mockResolvedValue({
      session_id: 'session-99',
      queue_size: 2,
      first_card: sampleCard,
    });
    gateway.stats.mockResolvedValue(sampleStats);

    await controller.start();
    controller.reset();

    expect(controller.getSnapshot()).toEqual({ status: 'idle', queueSize: 0 });
  });
});
