import { sessionGateway } from '../clients/sessionGateway';
import type { CardSummary, ReviewGrade, SessionStats } from '../types/gateway';

type SessionStoreState = {
  sessionId?: string;
  queue: CardSummary[];
  currentCard?: CardSummary;
  stats?: SessionStats;
  start: (userId: string) => Promise<void>;
  submitGrade: (grade: ReviewGrade, latencyMs: number) => Promise<void>;
  nextCard: (card?: CardSummary) => void;
};

type InternalState = Omit<SessionStoreState, 'start' | 'submitGrade' | 'nextCard'>;

type Listener = (state: SessionStoreState) => void;

const listeners = new Set<Listener>();

const baseState: InternalState = {
  sessionId: undefined,
  queue: [],
  currentCard: undefined,
  stats: undefined,
};

let state: SessionStoreState = {
  ...baseState,
  start: async (userId: string) => {
    const [session, sessionStats] = await Promise.all([
      sessionGateway.startSession(userId),
      sessionGateway.stats(),
    ]);
    setState({
      sessionId: session.session_id,
      queue: [],
      currentCard: session.first_card,
      stats: sessionStats,
    });
  },
  submitGrade: async (gradeValue: ReviewGrade, latencyMs: number) => {
    if (!state.currentCard) {
      return;
    }
    const result = await sessionGateway.grade(state.currentCard.card_id, gradeValue, latencyMs);
    state.nextCard(result.next_card);
    const updatedStats = await sessionGateway.stats();
    setState({ stats: updatedStats });
  },
  nextCard: (card?: CardSummary) => {
    setState({ currentCard: card });
  },
};

function setState(partial: Partial<InternalState>): void {
  state = { ...state, ...partial };
  listeners.forEach((listener) => {
    listener(state);
  });
}

export const sessionStore = {
  getState: (): SessionStoreState => state,
  setState: (partial: Partial<InternalState>) => {
    setState(partial);
  },
  subscribe: (listener: Listener) => {
    listeners.add(listener);
    return () => {
      listeners.delete(listener);
    };
  },
};
