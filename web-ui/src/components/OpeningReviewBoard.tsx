import type { JSX } from 'react';
import { useEffect, useMemo, useRef, useState } from 'react';
import type { Move, Square } from 'chess.js';
import { Chess } from 'chess.js';

import type { CardSummary, ReviewGrade } from '../types/gateway';

import 'chessboard-element';
import type { ChessBoardElement } from 'chessboard-element/lib/chessboard-element';
import './OpeningReviewBoard.css';

type Props = {
  card: CardSummary;
  onResult: (grade: ReviewGrade, latencyMs: number) => void;
};

type DropDetail = {
  source: Square;
  target: Square;
  promotion?: string;
};

type DropEvent = CustomEvent<DropDetail | undefined>;

const GOOD_RESULT: ReviewGrade = 'Good';
const MISS_RESULT: ReviewGrade = 'Again';

const FILES = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'] as const;
const RANKS = ['8', '7', '6', '5', '4', '3', '2', '1'] as const;
const OVERLAY_SQUARES: Square[] = RANKS.flatMap((rank) =>
  FILES.map((file) => `${file}${rank}` as Square),
);
const OVERLAY_SQUARE_SET = new Set<Square>(OVERLAY_SQUARES);

export function OpeningReviewBoard({ card, onResult }: Props): JSX.Element {
  const boardRef = useRef<ChessBoardElement | null>(null);
  const gameRef = useRef(new Chess(card.position_fen));
  const expectedMovesRef = useRef<string[]>(card.expected_moves_uci ?? []);
  const startedAtRef = useRef<number>(performance.now());
  const teachingArrowRef = useRef<string | null>(extractTeachingArrow(card.meta));
  const errorTimeoutRef = useRef<number | null>(null);
  const [selectedSquare, setSelectedSquare] = useState<Square | null>(null);
  const [legalTargets, setLegalTargets] = useState<Square[]>([]);
  const [errorSquare, setErrorSquare] = useState<Square | null>(null);
  const selectedSquareRef = useRef<Square | null>(null);

  const setSelectedSquareState = (square: Square | null) => {
    setSelectedSquare(square);
    const board = boardRef.current;
    if (!board) {
      return;
    }

    if (square) {
      board.setAttribute('data-selected-square', square);
    } else {
      board.removeAttribute('data-selected-square');
    }
  };

  const setErrorSquareState = (square: Square | null) => {
    setErrorSquare(square);
    const board = boardRef.current;
    if (!board) {
      return;
    }

    if (square) {
      board.setAttribute('data-error-square', square);
    } else {
      board.removeAttribute('data-error-square');
    }
  };

  const lichessAnalysisUrl = useMemo(
    () => `https://lichess.org/analysis/standard/${encodeURIComponent(card.position_fen)}`,
    [card.position_fen],
  );

  useEffect(() => {
    selectedSquareRef.current = selectedSquare;
  }, [selectedSquare]);

  useEffect(() => {
    setSelectedSquareState(null);
    setLegalTargets([]);
    setErrorSquareState(null);

    if (errorTimeoutRef.current !== null) {
      window.clearTimeout(errorTimeoutRef.current);
      errorTimeoutRef.current = null;
    }
  }, [card]);

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

    const clearErrorHighlight = () => {
      if (errorTimeoutRef.current !== null) {
        window.clearTimeout(errorTimeoutRef.current);
        errorTimeoutRef.current = null;
      }
      setErrorSquareState(null);
    };

    const showErrorHighlight = (square: Square | null) => {
      if (!square) {
        return;
      }

      if (errorTimeoutRef.current !== null) {
        window.clearTimeout(errorTimeoutRef.current);
      }

      setErrorSquareState(square);
      errorTimeoutRef.current = window.setTimeout(() => {
        setErrorSquareState(null);
        errorTimeoutRef.current = null;
      }, 900);
    };

    const attemptMove = (detail: DropDetail) => {
      const move = applyMove(gameRef.current, detail);
      if (!move) {
        showErrorHighlight(detail.target);
        return false;
      }

      const uci = toUci(move);
      const grade = chooseGrade(uci, expectedMovesRef.current);
      const latency = Math.max(0, Math.round(performance.now() - startedAtRef.current));

      board.setAttribute('position', gameRef.current.fen());

      if (grade === GOOD_RESULT) {
        clearErrorHighlight();
        teachingArrowRef.current = null;
        board.removeAttribute('data-teaching-arrow');
      } else {
        showErrorHighlight(detail.target);
        updateTeachingArrow(board, teachingArrowRef.current);
      }

      onResult(grade, latency);
      return true;
    };

    const handleDrop = (event: Event) => {
      const { detail } = event as DropEvent;
      if (!detail) {
        return;
      }

      if (attemptMove(detail)) {
        setSelectedSquareState(null);
        setLegalTargets([]);
      }
    };

    const handleBoardClick = (event: Event) => {
      const square = extractSquareFromEvent(event);
      if (!square) {
        return;
      }

      const selected = selectedSquareRef.current;

      if (selected) {
        if (square === selected) {
          setSelectedSquareState(null);
          setLegalTargets([]);
          return;
        }

        const moves: Move[] = gameRef.current.moves({ square: selected, verbose: true });
        const targetMove = moves.find((move) => move.to === square);

        if (!targetMove) {
          showErrorHighlight(square);
          return;
        }

        const detail: DropDetail = { source: selected, target: square };
        const promotion = targetMove.promotion;
        if (typeof promotion === 'string') {
          detail.promotion = promotion;
        }

        if (attemptMove(detail)) {
          setSelectedSquareState(null);
          setLegalTargets([]);
        }

        return;
      }

      const moves: Move[] = gameRef.current.moves({ square, verbose: true });
      if (moves.length === 0) {
        showErrorHighlight(square);
        return;
      }

      setSelectedSquareState(square);
      setLegalTargets(moves.map((move) => move.to));
    };

    board.addEventListener('drop', handleDrop);
    board.addEventListener('click', handleBoardClick);
    return () => {
      board.removeEventListener('drop', handleDrop);
      board.removeEventListener('click', handleBoardClick);
      clearErrorHighlight();
    };
  }, [card, onResult]);

  const legalTargetSet = useMemo(() => new Set<Square>(legalTargets), [legalTargets]);

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
      <div className="opening-review-board__board-wrapper" style={{ width: 'min(90vw, 560px)' }}>
        <chess-board
          data-testid="opening-review-board"
          ref={boardRef}
          className="opening-review-board__board"
          position={card.position_fen}
        />
        <div className="opening-review-board__overlay" data-testid="opening-review-board-overlay">
          {OVERLAY_SQUARES.map((square) => {
            const isSelected = square === selectedSquare;
            const isTarget = legalTargetSet.has(square);
            const isError = square === errorSquare;

            const states = [
              isSelected ? 'opening-review-board__overlay-square--selected' : '',
              isTarget ? 'opening-review-board__overlay-square--target' : '',
              isError ? 'opening-review-board__overlay-square--error' : '',
            ].filter(Boolean);

            const className = ['opening-review-board__overlay-square', ...states].join(' ');

            return (
              <div
                key={square}
                aria-hidden="true"
                className={className}
                data-overlay-square={square}
              />
            );
          })}
        </div>
      </div>
    </div>
  );
}

function extractSquareFromEvent(event: Event): Square | null {
  const path = event.composedPath();
  for (const target of path) {
    if (target instanceof HTMLElement) {
      const squareAttribute = target.dataset.square;
      if (isSquare(squareAttribute)) {
        return squareAttribute;
      }
    }
  }

  return null;
}

function applyMove(game: Chess, detail: DropDetail): Move | null {
  return game.move({ from: detail.source, to: detail.target, promotion: detail.promotion ?? 'q' });
}

function isSquare(value: unknown): value is Square {
  if (typeof value !== 'string') {
    return false;
  }

  const maybeSquare = value as Square;
  return OVERLAY_SQUARE_SET.has(maybeSquare);
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
