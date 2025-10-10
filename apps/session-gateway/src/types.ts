/* c8 ignore file */

/**
 * Review grades supported by the scheduler API.
 */
export type ReviewGrade = 'Again' | 'Hard' | 'Good' | 'Easy';

/**
 * Minimal card payload returned to session clients.
 */
export interface CardSummary {
  card_id: string;
  kind: 'Opening' | 'Tactic';
  position_fen: string;
  prompt: string;
  meta?: Record<string, string | number>;
}

/**
 * Aggregated statistics for an in-flight study session.
 */
export interface SessionStats {
  reviews_today: number;
  accuracy: number;
  avg_latency_ms: number;
}

/**
 * Contract implemented by scheduler clients consumed by the gateway.
 */
export interface SchedulerClient {
  fetchQueue(userId: string): Promise<CardSummary[]>;
  gradeCard(input: {
    sessionId: string;
    cardId: string;
    grade: ReviewGrade;
    latencyMs: number;
  }): Promise<CardSummary | null>;
}

/**
 * Persistence abstraction used to store in-progress session state.
 */
export interface SessionStore<T> {
  create(sessionId: string, value: T): Promise<void>;
  get(sessionId: string): Promise<T | undefined>;
  update(sessionId: string, updater: (current: T) => T): Promise<T>;
  delete(sessionId: string): Promise<void>;
}

/**
 * In-memory representation of a learner's active session.
 */
export interface SessionState {
  sessionId: string;
  userId: string;
  queue: CardSummary[];
  currentCard: CardSummary | null;
  stats: SessionStats;
  totalLatency: number;
}

/**
 * Dependencies required to construct the session gateway services.
 */
export interface GatewayDependencies {
  schedulerClient: SchedulerClient;
  sessionStore: SessionStore<SessionState>;
  logger?: {
    info(message: string, context?: Record<string, unknown>): void;
    warn(message: string, context?: Record<string, unknown>): void;
    error(message: string, context?: Record<string, unknown>): void;
  };
  clock?: () => Date;
}
