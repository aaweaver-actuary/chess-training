import { act } from 'react';
import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { Chess } from 'chess.js';
import type { Move } from 'chess.js';

import type { CardSummary } from '../../types/gateway';
import { OpeningReviewBoard } from '../OpeningReviewBoard';

describe('OpeningReviewBoard', () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.useRealTimers();
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

  const startingPosition: CardSummary = {
    card_id: 'starting',
    kind: 'Opening',
    position_fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
    prompt: 'Make a classical first move.',
    expected_moves_uci: ['e2e4'],
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

    act(() => {
      board.dispatchEvent(
        new CustomEvent('drop', {
          detail: { source: 'c1', target: 'g5', piece: 'wB' },
        }),
      );
    });

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

    act(() => {
      board.dispatchEvent(
        new CustomEvent('drop', {
          detail: { source: 'g1', target: 'f3', piece: 'wN' },
        }),
      );
    });

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

    act(() => {
      board.dispatchEvent(new CustomEvent('drop'));
    });

    expect(onResult).not.toHaveBeenCalled();
    expect(board).toHaveAttribute('position', baseCard.position_fen);
  });

  it('ignores invalid moves emitted by the board element', () => {
    const onResult = vi.fn();
    const moveSpy = vi.spyOn(Chess.prototype, 'move');
    moveSpy.mockImplementationOnce(() => null as unknown as Move);

    render(<OpeningReviewBoard card={baseCard} onResult={onResult} />);

    const board = screen.getByTestId('opening-review-board');

    act(() => {
      board.dispatchEvent(
        new CustomEvent('drop', {
          detail: { source: 'c1', target: 'c2' },
        }),
      );
    });

    expect(onResult).not.toHaveBeenCalled();
    expect(board).toHaveAttribute('position', baseCard.position_fen);
  });

  it('falls back to an empty expected move list when none are provided', () => {
    const onResult = vi.fn();
    const cardWithoutMoves: CardSummary = { ...baseCard, expected_moves_uci: undefined };
    render(<OpeningReviewBoard card={cardWithoutMoves} onResult={onResult} />);

    const board = screen.getByTestId('opening-review-board');

    act(() => {
      board.dispatchEvent(
        new CustomEvent('drop', {
          detail: { source: 'g1', target: 'f3' },
        }),
      );
    });

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

    act(() => {
      board.dispatchEvent(
        new CustomEvent('drop', {
          detail: { source: 'd7', target: 'd8', promotion: 'q' },
        }),
      );
    });

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

    act(() => {
      board.dispatchEvent(
        new CustomEvent('drop', {
          detail: { source: 'g1', target: 'f3', piece: 'wN' },
        }),
      );
    });

    expect(onResult).toHaveBeenCalledWith('Again', expect.any(Number));

    expect(board.getAttribute('data-error-square')).toBe('f3');
    expect(board.getAttribute('data-teaching-arrow')).toBe('e2e4');
  });

  it('selects a movable piece when it is clicked', async () => {
    const onResult = vi.fn();
    render(<OpeningReviewBoard card={startingPosition} onResult={onResult} />);

    const board = screen.getByTestId('opening-review-board');
    const e2 = await findBoardSquare(board, 'e2');

    fireEvent.click(e2);

    expect(board.getAttribute('data-selected-square')).toBe('e2');

    const overlaySquare = getOverlaySquare(board, 'e2');
    expect(overlaySquare).toHaveClass('opening-review-board__overlay-square--selected');
  });

  it('moves a piece when its legal destination square is clicked', async () => {
    const onResult = vi.fn();
    render(<OpeningReviewBoard card={startingPosition} onResult={onResult} />);

    const board = screen.getByTestId('opening-review-board');

    fireEvent.click(await findBoardSquare(board, 'e2'));
    fireEvent.click(await findBoardSquare(board, 'e4'));

    expect(board.getAttribute('position')).toBe(
      'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1',
    );
    expect(board.hasAttribute('data-selected-square')).toBe(false);

    const overlay = getOverlaySquare(board, 'e4');
    expect(overlay).not.toHaveClass('opening-review-board__overlay-square--error');
  });

  it('highlights an illegal destination when clicked before clearing it', async () => {
    const onResult = vi.fn();
    render(<OpeningReviewBoard card={startingPosition} onResult={onResult} />);

    const board = screen.getByTestId('opening-review-board');

    fireEvent.click(await findBoardSquare(board, 'e2'));
    fireEvent.click(await findBoardSquare(board, 'e5'));

    expect(board.getAttribute('data-error-square')).toBe('e5');

    const overlay = getOverlaySquare(board, 'e5');
    expect(overlay).toHaveClass('opening-review-board__overlay-square--error');

    await waitFor(
      () => {
        expect(board.hasAttribute('data-error-square')).toBe(false);
        expect(overlay).not.toHaveClass('opening-review-board__overlay-square--error');
      },
      { timeout: 1500 },
    );
  });

  it('indicates when a piece cannot be moved', async () => {
    const onResult = vi.fn();
    render(<OpeningReviewBoard card={startingPosition} onResult={onResult} />);

    const board = screen.getByTestId('opening-review-board');

    const e7 = await findBoardSquare(board, 'e7');
    fireEvent.click(e7);

    expect(board.hasAttribute('data-selected-square')).toBe(false);
    expect(board.getAttribute('data-error-square')).toBe('e7');

    const overlay = getOverlaySquare(board, 'e7');
    expect(overlay).toHaveClass('opening-review-board__overlay-square--error');

    await waitFor(
      () => {
        expect(board.hasAttribute('data-error-square')).toBe(false);
        expect(overlay).not.toHaveClass('opening-review-board__overlay-square--error');
      },
      { timeout: 1500 },
    );
  });
});

async function findBoardSquare(board: HTMLElement, square: string): Promise<HTMLElement> {
  return await waitFor(() => {
    const element = board.shadowRoot?.querySelector<HTMLElement>(`[data-square="${square}"]`);
    if (!element) {
      throw new Error(`Failed to locate square ${square}`);
    }

    return element;
  });
}

function getOverlaySquare(board: HTMLElement, square: string): HTMLElement {
  const wrapper = board.parentElement;
  if (!wrapper) {
    throw new Error('Board wrapper is missing');
  }

  const overlay = within(wrapper).getByTestId('opening-review-board-overlay');
  const element = overlay.querySelector<HTMLElement>(`[data-overlay-square="${square}"]`);

  if (!element) {
    throw new Error(`Failed to locate overlay square ${square}`);
  }

  return element;
}
