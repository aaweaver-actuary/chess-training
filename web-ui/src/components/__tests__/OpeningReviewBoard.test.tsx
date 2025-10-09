import { render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { Chess } from 'chess.js';
import type { Move } from 'chess.js';

import type { CardSummary } from '../../types/gateway';
import { OpeningReviewBoard } from '../OpeningReviewBoard';

describe('OpeningReviewBoard', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  const baseCard: CardSummary = {
    card_id: 'card-1',
    kind: 'Opening',
    position_fen: 'rn1qkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1',
    prompt: 'Play the natural developing move.',
    expected_moves_uci: ['c1g5'],
  };

  const italianStart: CardSummary = {
    card_id: 'italian-1',
    kind: 'Opening',
    position_fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
    prompt: 'Begin the Italian Game with the classical pawn thrust.',
    expected_moves_uci: ['e2e4'],
    meta: { teaching_move_uci: 'e2e4', line_reviews: 0 },
  };

  const italianSecondMove: CardSummary = {
    card_id: 'italian-2',
    kind: 'Opening',
    position_fen: 'rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2',
    prompt: 'Reinforce the centre with a developing knight move.',
    expected_moves_uci: ['g1f3'],
    meta: { teaching_move_uci: 'g1f3', line_reviews: 0 },
  };
  it('links to the Lichess analysis board for the current position', () => {
    const onResult = vi.fn();
    render(<OpeningReviewBoard card={baseCard} onResult={onResult} />);

    const shortcut = screen.getByRole('link', { name: /analyze this position on lichess/i });
    expect(shortcut).toHaveAttribute(
      'href',
      `https://lichess.org/analysis/standard/${encodeURIComponent(baseCard.position_fen)}`,
    );
  });

  it('reports success when the expected move is played', () => {
    const onResult = vi.fn();
    render(<OpeningReviewBoard card={baseCard} onResult={onResult} />);

    const board = screen.getByTestId('opening-review-board');

    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'c1', target: 'g5', piece: 'wB' },
      }),
    );

    expect(onResult).toHaveBeenCalledWith('Good', expect.any(Number));
    expect(board).toHaveAttribute(
      'position',
      'rn1qkbnr/ppp1pppp/8/3p2B1/3P4/8/PPP1PPPP/RN1QKBNR b KQkq - 1 1',
    );
  });

  it('reports a miss when an unexpected move is played', () => {
    const onResult = vi.fn();
    render(<OpeningReviewBoard card={baseCard} onResult={onResult} />);

    const board = screen.getByTestId('opening-review-board');

    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'g1', target: 'f3', piece: 'wN' },
      }),
    );

    expect(onResult).toHaveBeenCalledWith('Again', expect.any(Number));
    expect(board).toHaveAttribute(
      'position',
      'rn1qkbnr/ppp1pppp/8/3p4/3P4/5N2/PPP1PPPP/RNBQKB1R b KQkq - 1 1',
    );
  });

  it('ignores drop events that do not include move details', () => {
    const onResult = vi.fn();
    render(<OpeningReviewBoard card={baseCard} onResult={onResult} />);

    const board = screen.getByTestId('opening-review-board');

    board.dispatchEvent(new CustomEvent('drop'));

    expect(onResult).not.toHaveBeenCalled();
    expect(board).toHaveAttribute('position', baseCard.position_fen);
  });

  it('ignores invalid moves emitted by the board element', () => {
    const onResult = vi.fn();
    const moveSpy = vi.spyOn(Chess.prototype, 'move');
    moveSpy.mockImplementationOnce(() => null as unknown as Move);

    render(<OpeningReviewBoard card={baseCard} onResult={onResult} />);

    const board = screen.getByTestId('opening-review-board');

    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'c1', target: 'c2' },
      }),
    );

    expect(onResult).not.toHaveBeenCalled();
    expect(board).toHaveAttribute('position', baseCard.position_fen);
  });

  it('falls back to an empty expected move list when none are provided', () => {
    const onResult = vi.fn();
    const cardWithoutMoves: CardSummary = { ...baseCard, expected_moves_uci: undefined };
    render(<OpeningReviewBoard card={cardWithoutMoves} onResult={onResult} />);

    const board = screen.getByTestId('opening-review-board');

    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'g1', target: 'f3' },
      }),
    );

    expect(onResult).toHaveBeenCalledWith('Again', expect.any(Number));
  });

  it('includes the promotion piece when reporting a successful move', () => {
    const onResult = vi.fn();
    const promotionCard: CardSummary = {
      ...baseCard,
      position_fen: '4k3/3P4/8/8/8/8/8/4K3 w - - 0 1',
      expected_moves_uci: ['d7d8q'],
    };
    render(<OpeningReviewBoard card={promotionCard} onResult={onResult} />);

    const board = screen.getByTestId('opening-review-board');

    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'd7', target: 'd8', promotion: 'q' },
      }),
    );

    expect(onResult).toHaveBeenCalledWith('Good', expect.any(Number));
  });

  it('shows a teaching arrow for the first move of a new line', () => {
    const onResult = vi.fn();
    render(<OpeningReviewBoard card={italianStart} onResult={onResult} />);

    const board = screen.getByTestId('opening-review-board');

    expect(board.getAttribute('data-teaching-arrow')).toBe('e2e4');
  });

  it('omits the teaching arrow when the metadata does not provide a string value', () => {
    const onResult = vi.fn();
    const numericTeachingMeta: CardSummary = {
      ...baseCard,
      meta: { teaching_move_uci: 42 },
    };

    render(<OpeningReviewBoard card={numericTeachingMeta} onResult={onResult} />);

    const board = screen.getByTestId('opening-review-board');
    expect(board.hasAttribute('data-teaching-arrow')).toBe(false);
  });

  it('suppresses the teaching arrow once the line has existing reviews', () => {
    const onResult = vi.fn();
    const reviewedLineCard: CardSummary = {
      ...baseCard,
      meta: { teaching_move_uci: 'c2c4', line_reviews: 3 },
    };

    render(<OpeningReviewBoard card={reviewedLineCard} onResult={onResult} />);

    const board = screen.getByTestId('opening-review-board');
    expect(board.hasAttribute('data-teaching-arrow')).toBe(false);
  });

  it('updates the teaching arrow when presenting the follow-up move', () => {
    const onResult = vi.fn();
    const { rerender } = render(<OpeningReviewBoard card={italianStart} onResult={onResult} />);

    let board = screen.getByTestId('opening-review-board');
    expect(board.getAttribute('data-teaching-arrow')).toBe('e2e4');

    rerender(<OpeningReviewBoard card={italianSecondMove} onResult={onResult} />);

    board = screen.getByTestId('opening-review-board');
    expect(board.getAttribute('data-teaching-arrow')).toBe('g1f3');
  });

  it('marks the mistaken square and restores the teaching arrow after an incorrect move', () => {
    const onResult = vi.fn();
    render(<OpeningReviewBoard card={italianStart} onResult={onResult} />);

    const board = screen.getByTestId('opening-review-board');

    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'g1', target: 'f3', piece: 'wN' },
      }),
    );

    expect(onResult).toHaveBeenCalledWith('Again', expect.any(Number));

    expect(board.getAttribute('data-error-square')).toBe('f3');
    expect(board.getAttribute('data-teaching-arrow')).toBe('e2e4');
  });
});
