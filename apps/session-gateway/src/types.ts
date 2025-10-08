/* c8 ignore file */

export type ReviewGrade = 'Again' | 'Hard' | 'Good' | 'Easy';

export interface CardSummary {
  card_id: string;
  kind: 'Opening' | 'Tactic';
  position_fen: string;
  prompt: string;
  meta?: Record<string, string | number>;
}

export interface SessionStats {
  reviews_today: number;
  accuracy: number;
  avg_latency_ms: number;
}

export interface SchedulerClient {
  fetchQueue(userId: string): Promise<CardSummary[]>;
  gradeCard(input: {
    sessionId: string;
    cardId: string;
    grade: ReviewGrade;
    latencyMs: number;
  }): Promise<CardSummary | null>;
}

export interface SessionStore<T> {
  create(sessionId: string, value: T): Promise<void>;
  get(sessionId: string): Promise<T | undefined>;
  update(sessionId: string, updater: (current: T) => T): Promise<T>;
  delete(sessionId: string): Promise<void>;
}

export interface SessionState {
  sessionId: string;
  userId: string;
  queue: CardSummary[];
  currentCard: CardSummary | null;
  stats: SessionStats;
  totalLatency: number;
}

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
