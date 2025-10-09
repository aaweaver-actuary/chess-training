import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { BlankBoardPage } from '../BlankBoardPage';

describe('BlankBoardPage', () => {
  const renderPage = () => render(<BlankBoardPage />);

  it('renders an interactive chess board without additional chrome', () => {
    renderPage();

    const board = screen.getByTestId('sandbox-board');
    const shell = screen.getByRole('main', { name: /Sandbox board/i });

    expect(board).toBeInTheDocument();
    expect(board).toHaveAttribute('position', 'start');
    expect(board).toHaveAttribute('draggable-pieces', 'true');
    expect(board).toHaveStyle({ width: 'min(100vw, 100vh)' });
    expect(board).toHaveStyle({ height: 'min(100vw, 100vh)' });
    expect(shell).toHaveStyle({ position: 'fixed' });
    expect(shell).toHaveStyle({ inset: '0px' });
    expect(screen.queryByRole('heading', { name: /Sandbox Board/i })).not.toBeInTheDocument();
    expect(screen.queryByRole('link', { name: /back to dashboard/i })).not.toBeInTheDocument();
  });

  it('updates the board when a legal move is made', () => {
    renderPage();

    const board = screen.getByTestId('sandbox-board');
    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'e2', target: 'e4' },
      }),
    );

    expect(board).toHaveAttribute('position', 'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1');
  });

  it('restores the previous position when an illegal move is attempted', () => {
    renderPage();

    const board = screen.getByTestId('sandbox-board');
    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'e2', target: 'e5' },
      }),
    );

    expect(board).toHaveAttribute('position', 'start');
  });
});
