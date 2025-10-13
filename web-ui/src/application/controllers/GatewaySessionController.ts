import type {
  CardSummary,
  ReviewGrade,
  SessionStats,
  StartSessionResponse,
} from '../../types/gateway';
import type { SessionController, SessionSnapshot } from './SessionController';

export type SessionGateway = {
  startSession(userId: string): Promise<StartSessionResponse>;
  grade(
    sessionId: string,
    cardId: string,
    gradeValue: ReviewGrade,
    latencyMs: number,
  ): Promise<{ next_card?: CardSummary; stats?: SessionStats }>;
  stats(sessionId: string): Promise<SessionStats>;
};

const IDLE_SNAPSHOT: SessionSnapshot = {
  status: 'idle',
  queueSize: 0,
  currentCard: undefined,
  sessionId: undefined,
  stats: undefined,
  lastGrade: undefined,
  error: undefined,
};

const unknownError = (message: string, cause: unknown): string => {
  if (cause instanceof Error) {
    return `${message}: ${cause.message}`;
  }
  return message;
};

export class GatewaySessionController implements SessionController {
  private readonly gateway: SessionGateway;

  private snapshot: SessionSnapshot = { ...IDLE_SNAPSHOT };

  private listeners = new Set<(snapshot: SessionSnapshot) => void>();

  public constructor(gateway: SessionGateway) {
    this.gateway = gateway;
  }

  public getSnapshot(): SessionSnapshot {
    return this.snapshot;
  }

  public subscribe(listener: (snapshot: SessionSnapshot) => void): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  public async start(userId: string): Promise<void> {
    await this.startInternal(userId);
  }

  public async startDemo(): Promise<void> {
    await this.startInternal('demo-user');
  }

  public async submitGrade(grade: ReviewGrade, latencyMs: number): Promise<void> {
    const { currentCard, sessionId } = this.snapshot;
    if (!currentCard || !sessionId) {
      return;
    }

    this.updateSnapshot({ status: 'submittingGrade', lastGrade: grade, error: undefined });

    try {
      const response = await this.gateway.grade(sessionId, currentCard.card_id, grade, latencyMs);
      const stats =
        response.stats ?? (await this.safeLoadStats(sessionId)) ?? this.snapshot.stats;

      this.updateSnapshot({
        status: 'active',
        currentCard: response.next_card,
        stats,
      });
    } catch (error) {
      this.updateSnapshot({
        status: 'error',
        error: unknownError('Failed to submit grade', error),
      });
    }
  }

  public async preloadNext(): Promise<void> {
    // Future implementation can trigger background fetches.
    return Promise.resolve();
  }

  public reset(): void {
    this.updateSnapshot({ ...IDLE_SNAPSHOT });
  }

  private async startInternal(userId: string): Promise<void> {
    this.updateSnapshot({ status: 'loading', error: undefined });

    try {
      const session = await this.gateway.startSession(userId);

      this.updateSnapshot({
        status: 'active',
        sessionId: session.session_id,
        currentCard: session.first_card,
        queueSize: session.queue_size,
        stats: undefined,
      });

      const stats = await this.safeLoadStats(session.session_id);
      if (stats) {
        this.updateSnapshot({ stats });
      }
    } catch (error) {
      this.updateSnapshot({
        status: 'error',
        error: unknownError('Failed to start session', error),
      });
    }
  }

  private async safeLoadStats(sessionId: string): Promise<SessionStats | undefined> {
    try {
      return await this.gateway.stats(sessionId);
    } catch (error) {
      this.updateSnapshot({
        error: unknownError('Failed to load session stats', error),
      });
      return undefined;
    }
  }

  private updateSnapshot(patch: Partial<SessionSnapshot>): void {
    this.snapshot = { ...this.snapshot, ...patch };
    for (const listener of this.listeners) {
      listener(this.snapshot);
    }
  }
}
