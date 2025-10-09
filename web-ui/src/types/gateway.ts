/* c8 ignore file */

export type ReviewGrade = 'Again' | 'Hard' | 'Good' | 'Easy';

export type CardKind = 'Opening' | 'Tactic';

export type CardSummary = {
  card_id: string;
  kind: CardKind;
  position_fen: string;
  prompt: string;
  expected_moves_uci?: string[];
  pv_uci?: string[];
  meta?: Record<string, string | number>;
};

export type SessionStats = {
  reviews_today: number;
  accuracy: number;
  avg_latency_ms: number;
  due_count: number;
  completed_count: number;
};

export type StartSessionResponse = {
  session_id: string;
  queue_size: number;
  first_card: CardSummary;
};
