import type {
  CardSummary,
  ReviewGrade,
  SessionStats,
} from '../../types/gateway';

export type SessionStatus =
  | 'idle'
  | 'loading'
  | 'active'
  | 'submittingGrade'
  | 'completed'
  | 'error';

export type SessionSnapshot = {
  status: SessionStatus;
  sessionId?: string;
  currentCard?: CardSummary;
  queueSize: number;
  stats?: SessionStats;
  lastGrade?: ReviewGrade;
  error?: string;
};

export interface SessionController {
  getSnapshot(): SessionSnapshot;
  subscribe(listener: (snapshot: SessionSnapshot) => void): () => void;
  start(): Promise<void>;
  startDemo(): Promise<void>;
  submitGrade(grade: ReviewGrade, latencyMs: number): Promise<void>;
  preloadNext(): Promise<void>;
  reset(): void;
}
