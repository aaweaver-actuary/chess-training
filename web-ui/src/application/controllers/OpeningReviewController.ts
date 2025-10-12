import type { ReviewGrade } from '../../types/gateway';

export type ReviewMove = {
  san: string;
  uci: string;
  from: string;
  to: string;
};

export type OpeningReviewStatus =
  | 'idle'
  | 'preparing'
  | 'awaitingMove'
  | 'evaluating'
  | 'completed'
  | 'error';

export type OpeningReviewSnapshot = {
  status: OpeningReviewStatus;
  activeLineId?: string;
  boardFen: string;
  lastMove?: ReviewMove;
  expectedMoves: ReviewMove[];
  attemptedMoves: ReviewMove[];
  latencyMs?: number;
  error?: string;
};

export type OpeningReviewEvent =
  | { type: 'move'; move: ReviewMove }
  | { type: 'status'; status: OpeningReviewStatus }
  | { type: 'error'; message: string };

export interface OpeningReviewController {
  getSnapshot(): OpeningReviewSnapshot;
  selectSquare(square: string): void;
  dropPiece(from: string, to: string): void;
  submitGrade(grade: ReviewGrade): Promise<void>;
  loadLine(lineId: string): Promise<void>;
  reset(): void;
  subscribe(listener: (snapshot: OpeningReviewSnapshot, event?: OpeningReviewEvent) => void): () => void;
}
