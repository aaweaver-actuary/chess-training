import type {
  CardSummary,
  ReviewGrade,
  SessionStats,
  StartSessionResponse,
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

type SessionGateway = {
  startSession(userId: string): Promise<StartSessionResponse>;
  grade(
    cardId: string,
    gradeValue: ReviewGrade,
    latencyMs: number,
  ): Promise<{ next_card?: CardSummary }>;
  stats(): Promise<SessionStats>;
};

export type DemoSessionPlan = {
  sessionId: string;
  cards: CardSummary[];
  stats: SessionStats;
};

export type SessionControllerDependencies = {
  gateway: SessionGateway;
  defaultUserId?: string;
  demoSession?: DemoSessionPlan;
  onError?: (error: unknown) => void;
};

export type SessionSnapshotListener = (snapshot: SessionSnapshot) => void;

type ControllerMode = 'idle' | 'live' | 'demo';

type SnapshotUpdater =
  | Partial<SessionSnapshot>
  | ((snapshot: SessionSnapshot) => SessionSnapshot);

const serializeError = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === 'string') {
    return error;
  }

  try {
    return JSON.stringify(error);
  } catch (_jsonError) {
    void _jsonError;
  }

  return 'Unknown session error';
};

const initialSnapshot = (): SessionSnapshot => ({
  status: 'idle',
  queueSize: 0,
});

const applySnapshot = (
  snapshot: SessionSnapshot,
  update: SnapshotUpdater,
): SessionSnapshot => {
  if (typeof update === 'function') {
    return update(snapshot);
  }

  return { ...snapshot, ...update } satisfies SessionSnapshot;
};

export const createSessionController = ({
  gateway,
  defaultUserId,
  demoSession,
  onError,
}: SessionControllerDependencies): SessionController => {
  let snapshot = initialSnapshot();
  let mode: ControllerMode = 'idle';
  const listeners = new Set<SessionSnapshotListener>();
  let liveQueueSize = 0;
  let demoQueue: CardSummary[] = [];

  const notify = (update: SnapshotUpdater) => {
    snapshot = applySnapshot(snapshot, update);
    listeners.forEach((listener) => listener(snapshot));
  };

  const fail = (error: unknown) => {
    onError?.(error);
    notify({ status: 'error', error: serializeError(error) });
  };

  const resetInternals = () => {
    mode = 'idle';
    liveQueueSize = 0;
    demoQueue = [];
    snapshot = initialSnapshot();
  };

  const startLive = async () => {
    notify({ status: 'loading', error: undefined });

    try {
      const userId = defaultUserId ?? 'demo-user';
      const [session, stats] = await Promise.all([
        gateway.startSession(userId),
        gateway.stats(),
      ]);

      mode = 'live';
      liveQueueSize = Math.max(0, session.queue_size);
      demoQueue = [];

      notify({
        status: 'active',
        sessionId: session.session_id,
        currentCard: session.first_card,
        queueSize: liveQueueSize,
        stats,
        lastGrade: undefined,
        error: undefined,
      });
    } catch (error) {
      resetInternals();
      fail(error);
    }
  };

  const advanceDemo = (grade: ReviewGrade) => {
    const nextCard = demoQueue.shift();
    if (nextCard) {
      notify({
        status: 'active',
        currentCard: nextCard,
        queueSize: demoQueue.length + 1,
        lastGrade: grade,
        error: undefined,
      });
      return;
    }

    notify({
      status: 'completed',
      currentCard: undefined,
      queueSize: 0,
      lastGrade: grade,
      error: undefined,
    });
  };

  const advanceLive = async (
    cardId: string,
    grade: ReviewGrade,
    latencyMs: number,
  ) => {
    try {
      const gradeResult = await gateway.grade(cardId, grade, latencyMs);
      const stats = await gateway.stats();

      liveQueueSize = Math.max(0, liveQueueSize - 1);
      const nextCard = gradeResult.next_card;

      if (nextCard) {
        liveQueueSize = Math.max(liveQueueSize, 1);
        notify({
          status: 'active',
          currentCard: nextCard,
          queueSize: liveQueueSize,
          stats,
          lastGrade: grade,
          error: undefined,
        });
        return;
      }

      notify({
        status: 'completed',
        currentCard: undefined,
        queueSize: 0,
        stats,
        lastGrade: grade,
        error: undefined,
      });
    } catch (error) {
      fail(error);
    }
  };

  return {
    getSnapshot: () => snapshot,
    subscribe(listener) {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    async start() {
      await startLive();
    },
    async startDemo() {
      if (!demoSession || demoSession.cards.length === 0) {
        fail('Demo session is not available');
        return;
      }

      mode = 'demo';
      demoQueue = demoSession.cards.slice(1);
      liveQueueSize = demoSession.cards.length;

      notify({
        status: 'active',
        sessionId: demoSession.sessionId,
        currentCard: demoSession.cards[0],
        queueSize: liveQueueSize,
        stats: demoSession.stats,
        lastGrade: undefined,
        error: undefined,
      });
    },
    async submitGrade(grade, latencyMs) {
      const activeCard = snapshot.currentCard;
      if (!activeCard) {
        return;
      }

      notify({ status: 'submittingGrade', lastGrade: grade, error: undefined });

      if (mode === 'demo') {
        advanceDemo(grade);
        return;
      }

      await advanceLive(activeCard.card_id, grade, latencyMs);
    },
    async preloadNext() {
      if (mode === 'demo') {
        return;
      }
      // Placeholder for future queue preloading.
    },
    reset() {
      resetInternals();
    },
  };
};
