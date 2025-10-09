import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';

import type { CardSummary } from '../../types/gateway';
import { OpeningReviewPage } from '../OpeningReviewPage';

describe('OpeningReviewPage', () => {
  const card: CardSummary = {
    card_id: 'card-1',
    kind: 'Opening',
    position_fen: 'rn1qkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1',
    prompt: 'Play the natural developing move.',
    expected_moves_uci: ['c1g5'],
  };

  it('renders the board and grade controls when a card is available', async () => {
    const user = userEvent.setup();
    const handleGrade = vi.fn();
    const handleResult = vi.fn();

    render(
      <MemoryRouter>
        <OpeningReviewPage
          card={card}
          onGrade={handleGrade}
          onBoardResult={handleResult}
          backPath="/dashboard"
        />
      </MemoryRouter>,
    );

    expect(screen.getByRole('link', { name: /Back to Dashboard/i })).toHaveAttribute(
      'href',
      '/dashboard',
    );

    await user.click(screen.getByRole('button', { name: 'Good' }));
    expect(handleGrade).toHaveBeenCalledWith('Good');

    const board = screen.getByTestId('opening-review-board');
    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'c1', target: 'g5' },
      }),
    );
    expect(handleResult).toHaveBeenCalledWith('Good', expect.any(Number));
  });

  it('shows an empty state when there is no card to review', () => {
    render(
      <MemoryRouter>
        <OpeningReviewPage onGrade={vi.fn()} onBoardResult={vi.fn()} backPath="/dashboard" />
      </MemoryRouter>,
    );

    expect(screen.getByText('No opening card available right now.')).toBeInTheDocument();
  });
});
