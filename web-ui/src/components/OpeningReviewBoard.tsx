import { useEffect, useRef } from 'react';
import type { Move } from 'chess.js';
import { Chess } from 'chess.js';

import type { CardSummary, ReviewGrade } from '../types/gateway';

import 'chessboard-element';

type Props = {
  card: CardSummary;
  onResult: (grade: ReviewGrade, latencyMs: number) => void;
};

type DropDetail = {
  source: string;
  target: string;
  promotion?: string;
};

type DropEvent = CustomEvent<DropDetail | undefined>;

const GOOD_RESULT: ReviewGrade = 'Good';
const MISS_RESULT: ReviewGrade = 'Again';

export function OpeningReviewBoard({ card, onResult }: Props): JSX.Element {
  const boardRef = useRef<HTMLElement | null>(null);
  const gameRef = useRef(new Chess(card.position_fen));
  const expectedMovesRef = useRef<string[]>(card.expected_moves_uci ?? []);
  const startedAtRef = useRef<number>(performance.now());
  const teachingArrowRef = useRef<string | null>(extractTeachingArrow(card.meta));

  useEffect(() => {
    expectedMovesRef.current = card.expected_moves_uci ?? [];
    startedAtRef.current = performance.now();
    const game = new Chess(card.position_fen);
    gameRef.current = game;
    teachingArrowRef.current = extractTeachingArrow(card.meta);

    const board = boardRef.current;
    if (!board) {
      return;
    }

    board.setAttribute('position', card.position_fen);
    updateTeachingArrow(board, teachingArrowRef.current);
    board.removeAttribute('data-error-square');

    const handleDrop = (event: Event) => {
      const { detail } = event as DropEvent;
      if (!detail) {
        return;
      }

      const move = applyMove(gameRef.current, detail);
      if (!move) {
        return;
      }

      const uci = toUci(move);
      const grade = chooseGrade(uci, expectedMovesRef.current);
      const latency = Math.max(0, Math.round(performance.now() - startedAtRef.current));

      board.setAttribute('position', gameRef.current.fen());

      if (grade === GOOD_RESULT) {
        board.removeAttribute('data-error-square');
        teachingArrowRef.current = null;
        board.removeAttribute('data-teaching-arrow');
      } else {
        board.setAttribute('data-error-square', detail.target);
        updateTeachingArrow(board, teachingArrowRef.current);
      }

      onResult(grade, latency);
    };

    board.addEventListener('drop', handleDrop);
    return () => {
      board.removeEventListener('drop', handleDrop);
    };
  }, [card, onResult]);

  return (
    <chess-board
      data-testid="opening-review-board"
      ref={boardRef}
      style={{ width: 'min(90vw, 560px)' }}
      position={card.position_fen}
    />
  );
}

function applyMove(game: Chess, detail: DropDetail): Move | null {
  return game.move({ from: detail.source, to: detail.target, promotion: detail.promotion ?? 'q' });
}

function extractTeachingArrow(meta?: CardSummary['meta']): string | null {
  if (!meta) {
    return null;
  }

  const teachingValue = meta['teaching_move_uci'];
  if (typeof teachingValue !== 'string') {
    return null;
  }

  const lineReviewsValue = meta['line_reviews'];
  const lineReviews = typeof lineReviewsValue === 'number'
    ? lineReviewsValue
    : typeof lineReviewsValue === 'string'
    ? Number(lineReviewsValue)
    : NaN;

  if (!Number.isNaN(lineReviews) && lineReviews > 0) {
    return null;
  }

  return teachingValue;
}

function updateTeachingArrow(board: HTMLElement, teachingArrow: string | null): void {
  if (teachingArrow) {
    board.setAttribute('data-teaching-arrow', teachingArrow);
  } else {
    board.removeAttribute('data-teaching-arrow');
  }
}

function toUci(move: Move): string {
  const promotion = move.promotion ? move.promotion : '';
  return `${move.from}${move.to}${promotion}`;
}

function chooseGrade(uci: string, expectedMoves: string[]): ReviewGrade {
  return expectedMoves.includes(uci) ? GOOD_RESULT : MISS_RESULT;
}
