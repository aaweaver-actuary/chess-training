import { randomUUID } from 'crypto';
import { Broadcaster } from './broadcaster.js';
import {
  CardSummary,
  GatewayDependencies,
  ReviewGrade,
  SessionState,
  SessionStats,
} from './types.js';

const initialStats = (): SessionStats => ({
  reviews_today: 0,
  accuracy: 0,
  avg_latency_ms: 0,
});

const isCorrect = (grade: ReviewGrade) => grade !== 'Again';

const computeAccuracy = (correct: number, total: number) =>
  total === 0 ? 0 : correct / total;

const noop = () => undefined;

export interface SessionService {
  start(userId: string): Promise<{
    sessionId: string;
    queueSize: number;
    firstCard: CardSummary | null;
  }>;
  grade(input: {
    sessionId: string;
    cardId: string;
    grade: ReviewGrade;
    latencyMs: number;
  }): Promise<{ nextCard: CardSummary | null; stats: SessionStats }>;
  stats(sessionId: string): Promise<SessionStats | undefined>;
  end(sessionId: string): Promise<void>;
}

interface SessionServiceDependencies extends GatewayDependencies {
  broadcaster: Broadcaster;
}

const applyGrade = (
  current: SessionState,
  nextCard: CardSummary | null,
  grade: ReviewGrade,
  latencyMs: number,
) => {
  const totalReviews = current.stats.reviews_today + 1;
  const totalLatency = current.totalLatency + latencyMs;
  const correctReviews = current.stats.accuracy * current.stats.reviews_today;
  const accuracy = computeAccuracy(
    correctReviews + (isCorrect(grade) ? 1 : 0),
    totalReviews,
  );
  const avgLatency = Math.round(totalLatency / totalReviews);
  const stats: SessionStats = {
    reviews_today: totalReviews,
    accuracy,
    avg_latency_ms: avgLatency,
  };
  return { ...current, currentCard: nextCard, stats, totalLatency };
};

export const createSessionService = (
  deps: SessionServiceDependencies,
): SessionService => {
  const {
    schedulerClient,
    sessionStore,
    broadcaster,
    logger = { info: noop, warn: noop, error: noop },
  } = deps;

  return {
    async start(userId) {
      const sessionId = randomUUID();
      const queue = await schedulerClient.fetchQueue(userId);
      const firstCard = queue[0] ?? null;
      const state: SessionState = {
        sessionId,
        userId,
        queue,
        currentCard: firstCard,
        stats: initialStats(),
        totalLatency: 0,
      };
      await sessionStore.create(sessionId, state);
      logger.info('session-start', { sessionId, userId, queueSize: queue.length });
      return { sessionId, queueSize: queue.length, firstCard };
    },
    async grade({ sessionId, cardId, grade, latencyMs }) {
      const current = await sessionStore.get(sessionId);
      if (!current || !current.currentCard || current.currentCard.card_id !== cardId) {
        throw new Error('invalid-session');
      }
      const nextCard = await schedulerClient.gradeCard({
        sessionId,
        cardId,
        grade,
        latencyMs,
      });
      const updated = await sessionStore.update(sessionId, (state) =>
        applyGrade(state, nextCard, grade, latencyMs),
      );
      broadcaster.broadcast(sessionId, { type: 'UPDATE', card: nextCard });
      broadcaster.broadcast(sessionId, { type: 'STATS', stats: updated.stats });
      return { nextCard, stats: updated.stats };
    },
    async stats(sessionId) {
      const state = await sessionStore.get(sessionId);
      return state?.stats;
    },
    async end(sessionId) {
      await sessionStore.delete(sessionId);
      broadcaster.broadcast(sessionId, { type: 'SESSION_END', completed: true });
      logger.info('session-end', { sessionId });
    },
  };
};
