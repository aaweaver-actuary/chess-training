import { useEffect, useMemo, useRef } from 'react';
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
  const lichessAnalysisUrl = useMemo(
    () => `https://lichess.org/analysis/standard/${encodeURIComponent(card.position_fen)}`,
    [card.position_fen],
  );

  useEffect(() => {
    expectedMovesRef.current = card.expected_moves_uci ?? [];
    startedAtRef.current = performance.now();
    const game = new Chess(card.position_fen);
    gameRef.current = game;
    teachingArrowRef.current = extractTeachingArrow(card.meta);

    const board = boardRef.current;
    /* c8 ignore next 3 -- React always assigns the ref before this effect runs */
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
    <div className="opening-review-board">
      <a
        aria-label="Analyze this position on Lichess"
        className="floating-action lichess-shortcut"
        href={lichessAnalysisUrl}
        rel="noopener noreferrer"
        target="_blank"
      >
        <svg
          width="24"
          height="24"
          viewBox="0 0 24 24"
          fill="currentColor"
          xmlns="http://www.w3.org/2000/svg"
          aria-hidden="true"
          focusable="false"
          style={{ verticalAlign: 'middle' }}
        >
          <title>Analyze on Lichess</title>
          <path d="M7 2C7 2 8 4 8 6C8 8 6 10 6 12C6 14 8 16 10 16C12 16 14 14 14 12C14 10 12 8 12 6C12 4 13 2 13 2H7ZM10 18C8.34315 18 7 19.3431 7 21H13C13 19.3431 11.6569 18 10 18Z" />
        </svg>
      </a>
      <chess-board
        data-testid="opening-review-board"
        ref={boardRef}
        style={{ width: 'min(90vw, 560px)' }}
        className="opening-review-board__board"
        position={card.position_fen}
      />
    </div>
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
  const lineReviews = Number(lineReviewsValue);

  if (Number.isFinite(lineReviews) && lineReviews > 0) {
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
