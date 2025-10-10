import { randomUUID } from 'crypto';
const initialStats = () => ({
  reviews_today: 0,
  accuracy: 0,
  avg_latency_ms: 0,
});
const isCorrect = (grade) => grade !== 'Again';
const computeAccuracy = (correct, total) => (total === 0 ? 0 : correct / total);
const noop = () => undefined;
const applyGrade = (current, nextCard, grade, latencyMs) => {
  const totalReviews = current.stats.reviews_today + 1;
  const totalLatency = current.totalLatency + latencyMs;
  const correctReviews = current.stats.accuracy * current.stats.reviews_today;
  const accuracy = computeAccuracy(
    correctReviews + (isCorrect(grade) ? 1 : 0),
    totalReviews,
  );
  const avgLatency = Math.round(totalLatency / totalReviews);
  const stats = {
    reviews_today: totalReviews,
    accuracy,
    avg_latency_ms: avgLatency,
  };
  return { ...current, currentCard: nextCard, stats, totalLatency };
};
export const createSessionService = (deps) => {
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
      const state = {
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
      broadcaster.broadcast(sessionId, {
        type: 'UPDATE',
        card: nextCard,
        stats: updated.stats,
      });
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
