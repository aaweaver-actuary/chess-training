import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { BlankBoardPage } from '../BlankBoardPage';

// The duplication bug originally surfaced after dragging the same illegal move
// several times; attempting it three times covers that regression scenario.
const ILLEGAL_MOVE_ATTEMPTS = 3;

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
    expect(
      screen.queryByRole('heading', { name: /Sandbox Board/i }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole('link', { name: /back to dashboard/i }),
    ).not.toBeInTheDocument();
  });

  it('updates the board when a legal move is made', () => {
    renderPage();

    const board = screen.getByTestId('sandbox-board');
    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'e2', target: 'e4' },
      }),
    );

    expect(board).toHaveAttribute(
      'position',
      'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1',
    );
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

  it('prevents the same side from moving twice in a row', () => {
    renderPage();

    const board = screen.getByTestId('sandbox-board');
    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'e2', target: 'e4' },
      }),
    );

    expect(board).toHaveAttribute(
      'position',
      'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1',
    );

    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'g2', target: 'g4' },
      }),
    );

    expect(board).toHaveAttribute(
      'position',
      'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1',
    );
  });

  it('blocks pawn jumps that are not allowed by the rules of chess', () => {
    renderPage();

    const board = screen.getByTestId('sandbox-board');
    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'e2', target: 'e3' },
      }),
    );

    expect(board).toHaveAttribute(
      'position',
      'rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1',
    );

    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'e3', target: 'e5' },
      }),
    );

    expect(board).toHaveAttribute(
      'position',
      'rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1',
    );

    const shadowRoot = board.shadowRoot;
    if (!shadowRoot) {
      throw new Error('The chess board should expose a shadow root in tests.');
    }

    expect(
      shadowRoot.querySelector('[data-square="e5"] [part~="piece"]'),
    ).toBeNull();
  });

  it('prevents pawns from moving illegally without a capture', () => {
    renderPage();

    const board = screen.getByTestId('sandbox-board');
    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'e2', target: 'e4' },
      }),
    );

    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'e4', target: 'f4' },
      }),
    );

    expect(board).toHaveAttribute(
      'position',
      'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1',
    );
  });

  it('does not duplicate pieces after repeated illegal moves', async () => {
    renderPage();

    const board = screen.getByTestId('sandbox-board');
    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'a2', target: 'a4' },
      }),
    );

    const expectedPosition =
      'rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq - 0 1';

    expect(board).toHaveAttribute('position', expectedPosition);

    for (let attempt = 0; attempt < ILLEGAL_MOVE_ATTEMPTS; attempt += 1) {
      board.dispatchEvent(
        new CustomEvent('drop', {
          detail: { source: 'a4', target: 'a5' },
        }),
      );
    }

    expect(board).toHaveAttribute('position', expectedPosition);

    const shadowRoot = board.shadowRoot;
    if (!shadowRoot) {
      throw new Error('The chess board should expose a shadow root in tests.');
    }

    await waitFor(() => {
      expect(
        shadowRoot.querySelector('[data-square="a4"] [part~="piece"]'),
      ).not.toBeNull();
    });

    await waitFor(() => {
      expect(
        shadowRoot.querySelector('[data-square="a2"] [part~="piece"]'),
      ).toBeNull();
    });
  });

  it('allows pieces to be activated and moved with clicks', async () => {
    renderPage();

    const board = screen.getByTestId('sandbox-board');
    const shadowRoot = board.shadowRoot;
    if (!shadowRoot) {
      throw new Error('The chess board should expose a shadow root in tests.');
    }

    await waitFor(() => {
      expect(shadowRoot.querySelector('[data-square="e2"]')).not.toBeNull();
    });

    const e2 = shadowRoot.querySelector('[data-square="e2"]');
    const e4 = shadowRoot.querySelector('[data-square="e4"]');
    if (!e2 || !e4) {
      throw new Error('Expected squares were not found on the board.');
    }

    fireEvent.click(e2);

    expect(board).toHaveAttribute('data-active-square', 'e2');

    fireEvent.click(e4);

    expect(board).not.toHaveAttribute('data-active-square');
    expect(board).toHaveAttribute(
      'position',
      'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1',
    );
  });

  it('clears the selection when the same square is clicked twice', async () => {
    renderPage();

    const board = screen.getByTestId('sandbox-board');
    const shadowRoot = board.shadowRoot;
    if (!shadowRoot) {
      throw new Error('The chess board should expose a shadow root in tests.');
    }

    await waitFor(() => {
      expect(shadowRoot.querySelector('[data-square="e2"]')).not.toBeNull();
    });
    const e2 = shadowRoot.querySelector('[data-square="e2"]');
    if (!e2) {
      throw new Error('Expected square e2 to exist on the board.');
    }

    fireEvent.click(e2);
    expect(board).toHaveAttribute('data-active-square', 'e2');

    fireEvent.click(e2);
    expect(board).not.toHaveAttribute('data-active-square');
  });

  it('switches the selection to another friendly piece when clicked', async () => {
    renderPage();

    const board = screen.getByTestId('sandbox-board');
    const shadowRoot = board.shadowRoot;
    if (!shadowRoot) {
      throw new Error('The chess board should expose a shadow root in tests.');
    }

    await waitFor(() => {
      expect(shadowRoot.querySelector('[data-square="g1"]')).not.toBeNull();
      expect(shadowRoot.querySelector('[data-square="b1"]')).not.toBeNull();
    });
    const g1 = shadowRoot.querySelector('[data-square="g1"]');
    const b1 = shadowRoot.querySelector('[data-square="b1"]');
    if (!g1 || !b1) {
      throw new Error('Expected knight squares to exist on the board.');
    }

    fireEvent.click(g1);
    expect(board).toHaveAttribute('data-active-square', 'g1');

    fireEvent.click(b1);
    expect(board).toHaveAttribute('data-active-square', 'b1');
  });

  it('keeps the current selection active after an invalid move to an empty square', async () => {
    renderPage();

    const board = screen.getByTestId('sandbox-board');
    const shadowRoot = board.shadowRoot;
    if (!shadowRoot) {
      throw new Error('The chess board should expose a shadow root in tests.');
    }

    await waitFor(() => {
      expect(shadowRoot.querySelector('[data-square="g1"]')).not.toBeNull();
      expect(shadowRoot.querySelector('[data-square="g3"]')).not.toBeNull();
    });
    const g1 = shadowRoot.querySelector('[data-square="g1"]');
    const g3 = shadowRoot.querySelector('[data-square="g3"]');
    if (!g1 || !g3) {
      throw new Error('Expected squares were not found on the board.');
    }

    fireEvent.click(g1);
    expect(board).toHaveAttribute('data-active-square', 'g1');

    fireEvent.click(g3);
    expect(board).toHaveAttribute('data-active-square', 'g1');
  });

  it('requests a snapback when a drag-and-drop move is invalid', () => {
    renderPage();

    const board = screen.getByTestId('sandbox-board');
    const setAction = vi.fn();

    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'e2', target: 'e5', setAction },
      }),
    );

    expect(setAction).toHaveBeenCalledWith('snapback');
    expect(board).toHaveAttribute('position', 'start');
  });

  it('clears the selection if the original piece has moved externally', async () => {
    renderPage();

    const board = screen.getByTestId('sandbox-board');
    const shadowRoot = board.shadowRoot;
    if (!shadowRoot) {
      throw new Error('The chess board should expose a shadow root in tests.');
    }

    await waitFor(() => {
      expect(shadowRoot.querySelector('[data-square="e2"]')).not.toBeNull();
      expect(shadowRoot.querySelector('[data-square="e4"]')).not.toBeNull();
    });

    const e2 = shadowRoot.querySelector('[data-square="e2"]');
    const e4 = shadowRoot.querySelector('[data-square="e4"]');
    if (!e2 || !e4) {
      throw new Error('Expected squares were not found on the board.');
    }

    fireEvent.click(e2);
    expect(board).toHaveAttribute('data-active-square', 'e2');

    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'e2', target: 'e4' },
      }),
    );

    fireEvent.click(e4);

    await waitFor(() => {
      expect(board).not.toHaveAttribute('data-active-square');
    });
  });

  it('clears the selection when a click does not land on a square', async () => {
    renderPage();

    const board = screen.getByTestId('sandbox-board');
    const shadowRoot = board.shadowRoot;
    if (!shadowRoot) {
      throw new Error('The chess board should expose a shadow root in tests.');
    }

    await waitFor(() => {
      expect(shadowRoot.querySelector('[data-square="e2"]')).not.toBeNull();
    });

    const e2 = shadowRoot.querySelector('[data-square="e2"]');
    if (!e2) {
      throw new Error('The expected square was not found on the board.');
    }

    fireEvent.click(e2);
    expect(board).toHaveAttribute('data-active-square', 'e2');

    fireEvent.click(board);

    await waitFor(() => {
      expect(board).not.toHaveAttribute('data-active-square');
    });
  });
});
