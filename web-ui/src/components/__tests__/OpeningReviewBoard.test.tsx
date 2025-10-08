import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { CardSummary } from '../../types/gateway';
import { OpeningReviewBoard } from '../OpeningReviewBoard';

describe('OpeningReviewBoard', () => {
  const baseCard: CardSummary = {
    card_id: 'card-1',
    kind: 'Opening',
    position_fen: 'rn1qkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1',
    prompt: 'Play the natural developing move.',
    expected_moves_uci: ['c1g5'],
  };

  it('links to the Lichess analysis board for the current position', () => {
    const onResult = vi.fn();
    render(<OpeningReviewBoard card={baseCard} onResult={onResult} />);

    const shortcut = screen.getByRole('link', { name: /open position on lichess/i });
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
});
