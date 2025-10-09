import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { UserEvent } from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';

function createDefaultStats() {
  return {
    reviews_today: 3,
    accuracy: 0.75,
    avg_latency_ms: 1800,
    due_count: 12,
    completed_count: 9,
  };
}

vi.mock('./state/sessionStore', () => {
  const listeners = new Set<(state: unknown) => void>();
  const state = {
    sessionId: 's1',
    queue: [],
    currentCard: {
      card_id: 'c1',
      kind: 'Opening',
      position_fen: 'rn1qkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1',
      prompt: 'Play the move',
      expected_moves_uci: ['c1g5'],
    },
    stats: createDefaultStats(),
    start: vi.fn(),
    submitGrade: vi.fn(),
    nextCard: vi.fn(),
  };

  return {
    sessionStore: {
      getState: () => state,
      subscribe: (listener: (state: unknown) => void) => {
        listeners.add(listener);
        return () => listeners.delete(listener);
      },
    },
  };
});

import App from './App';
import { sessionStore } from './state/sessionStore';

const setupUser = (): UserEvent => userEvent.setup();

const mockedStore = sessionStore as unknown as {
  getState: () => {
    start: ReturnType<typeof vi.fn>;
    submitGrade: ReturnType<typeof vi.fn>;
    stats: ReturnType<typeof createDefaultStats> | undefined;
  } & Record<string, unknown>;
};

beforeEach(() => {
  const state = mockedStore.getState();
  state.start.mockClear();
  state.submitGrade.mockClear();
  state.stats = createDefaultStats();
  state.currentCard.kind = 'Opening';
});

describe('App', () => {
  it('starts the session on mount and renders live stats', async () => {
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });
    expect(screen.getByRole('heading', { name: /Daily Review Summary/i })).toBeInTheDocument();
    expect(screen.getByText('12')).toBeInTheDocument();
    expect(screen.getByText('9')).toBeInTheDocument();
  });

  it('submits a grade when clicking a grade button', async () => {
    const user = setupUser();
    render(
      <MemoryRouter initialEntries={['/review/opening']}>
        <App />
      </MemoryRouter>,
    );

    await user.click(screen.getByRole('button', { name: /Good/i }));

    expect(mockedStore.getState().submitGrade).toHaveBeenCalledWith('Good', expect.any(Number));
  });

  it('submits board results when the opening review board emits a move', async () => {
    render(
      <MemoryRouter initialEntries={['/review/opening']}>
        <App />
      </MemoryRouter>,
    );

    const board = await screen.findByTestId('opening-review-board');

    board.dispatchEvent(
      new CustomEvent('drop', {
        detail: { source: 'g1', target: 'h3' },
      }),
    );

    await waitFor(() => {
      expect(mockedStore.getState().submitGrade).toHaveBeenCalledWith('Again', expect.any(Number));
    });
  });

  it('shows an empty opening state when the current card is not an opening', async () => {
    mockedStore.getState().currentCard.kind = 'Tactic';

    render(
      <MemoryRouter initialEntries={['/review/opening']}>
        <App />
      </MemoryRouter>,
    );

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });

    expect(screen.getByText('No opening card available right now.')).toBeInTheDocument();
  });

  it('falls back to the baseline overview when stats are unavailable', async () => {
    mockedStore.getState().stats = undefined;
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });
    expect(screen.getByText('18')).toBeInTheDocument();
    expect(screen.getByText('11')).toBeInTheDocument();
  });

  it('treats zero due counts as fully complete', async () => {
    mockedStore.getState().stats = {
      ...createDefaultStats(),
      due_count: 0,
      completed_count: 0,
    };
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });
    const dueCard = screen.getByText('Due Today').closest('.metric-card');
    expect(dueCard).not.toBeNull();
    expect(within(dueCard as HTMLElement).getByText('0')).toBeInTheDocument();
    expect(screen.getByText(/100% complete/i)).toBeInTheDocument();
  });
  it('toggles the command console with keyboard shortcuts', async () => {
    const user = setupUser();
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });

    const launcher = screen.getByRole('button', { name: /open command console/i });

    const input = document.createElement('input');
    document.body.appendChild(input);
    input.focus();

    await user.keyboard('{Shift>}{;}');
    expect(screen.getByRole('button', { name: /open command console/i })).toBe(launcher);

    input.blur();
    input.remove();

    const editable = document.createElement('div');
    editable.contentEditable = 'true';
    Object.defineProperty(editable, 'isContentEditable', { value: true });
    document.body.appendChild(editable);
    editable.focus();

    fireEvent.keyDown(editable, { key: ';', shiftKey: true });
    expect(screen.getByRole('button', { name: /open command console/i })).toBe(launcher);

    editable.remove();

    await user.keyboard('{Shift>}{;}');

    await waitFor(() => {
      const closeButtons = screen.getAllByRole('button', { name: /close command console/i });
      expect(closeButtons.length).toBeGreaterThan(0);
    });

    await user.keyboard('{Escape}');

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /open command console/i })).toBeInTheDocument();
    });
  });

  it('opens and closes the command console with the launcher button', async () => {
    const user = setupUser();
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });

    const launcher = screen.getByRole('button', { name: /open command console/i });
    expect(launcher).toHaveAttribute('aria-expanded', 'false');

    await user.click(launcher);

    await waitFor(() => {
      const closeButtons = screen.getAllByRole('button', { name: /close command console/i });
      expect(closeButtons).toHaveLength(2);
    });

    const [toggleButton] = screen.getAllByRole('button', { name: /close command console/i });
    expect(toggleButton).toHaveAttribute('aria-expanded', 'true');

    await user.click(toggleButton);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /open command console/i })).toHaveAttribute(
        'aria-expanded',
        'false',
      );
    });
  });

  it('keeps the command console close button within the header so it remains visible', async () => {
    const user = setupUser();

    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    const launcher = screen.getByRole('button', { name: /open command console/i });
    await user.click(launcher);

    const consoleDialog = await screen.findByRole('dialog', { name: /command console/i });
    const closeButton = within(consoleDialog).getByRole('button', { name: /close command console/i });

    expect(closeButton).toHaveClass('command-console__close');
    expect(closeButton.closest('.command-console__header')).not.toBeNull();
  });

  it('opens the command console for key events dispatched from window targets', async () => {
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });

    const launcher = screen.getByRole('button', { name: /open command console/i });
    expect(launcher).toHaveAttribute('aria-expanded', 'false');

    act(() => {
      window.dispatchEvent(
        new KeyboardEvent('keydown', { key: ';', shiftKey: true, bubbles: true }),
      );
    });

    await waitFor(() => {
      const closeButtons = screen.getAllByRole('button', { name: /close command console/i });
      expect(closeButtons).toHaveLength(2);
    });
  });

  it('allows navigating between the dashboard and the opening review board', async () => {
    const user = setupUser();
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });

    const startReviewLink = screen.getByRole('link', { name: /Start opening review/i });
    await user.click(startReviewLink);

    expect(screen.getByRole('region', { name: /Opening review/i })).toBeInTheDocument();

    const backLink = screen.getByRole('link', { name: /Back to dashboard/i });
    await user.click(backLink);

    expect(screen.getByRole('heading', { name: /Daily Review Summary/i })).toBeInTheDocument();
  });

  it('allows importing a Danish Gambit line from the dashboard PGN tools', async () => {
    const user = setupUser();

    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });

    const hoverHandle = screen.getByLabelText(/open pgn import tools/i);
    fireEvent.pointerEnter(hoverHandle);

    const pasteOption = await screen.findByRole('button', { name: /paste pgn/i });
    await user.click(pasteOption);

    const pgnInput = await screen.findByLabelText(/pgn move input/i);
    await user.type(pgnInput, '1.e4 e5 2.d4 exd4 3.c3');

    const confirmButton = await screen.findByRole('button', {
      name: /Add to Danish Gambit \(White\)/i,
    });
    await user.click(confirmButton);

    const feedback = await screen.findByText(/Scheduled for/i);
    expect(feedback).toHaveTextContent(/Danish Gambit/i);

    const upcomingList = screen.getByRole('list', { name: /upcoming unlocks/i });
    await waitFor(() => {
      expect(within(upcomingList).getByText(/Danish Gambit \(White\)/i)).toBeInTheDocument();
    });

    expect(within(upcomingList).getByText(/Line: 1\.e4 e5 2\.d4 exd4 3\.c3/i)).toBeInTheDocument();

    await user.type(pgnInput, '1.e4 e5 2.d4 exd4 3.c3');

    const duplicateButton = await screen.findByRole('button', {
      name: /Add to Danish Gambit \(White\)/i,
    });
    await user.click(duplicateButton);

    const duplicateFeedback = await screen.findByText(/Already scheduled/i);
    expect(duplicateFeedback).toHaveTextContent(/Already scheduled/i);

    const danishEntries = within(upcomingList).getAllByText(/Danish Gambit \(White\)/i);
    expect(danishEntries).toHaveLength(1);

    await user.clear(pgnInput);
    await waitFor(() => {
      expect(pgnInput).toHaveValue('');
    });
  });

  it('opens the command console when the colon key is pressed', async () => {
    const user = setupUser();
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });

    expect(screen.queryByRole('dialog', { name: /Command console/i })).not.toBeInTheDocument();

    await user.keyboard(':');

    expect(screen.getByRole('dialog', { name: /Command console/i })).toBeInTheDocument();
  });

  it('focuses the command input when the console opens via the colon key', async () => {
    const user = setupUser();
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });

    await user.keyboard(':ADD');

    const input = await screen.findByRole('textbox', { name: /command input/i });
    expect(input).toHaveFocus();
    expect(input).toHaveValue('ADD');
  });

  it('closes the command console when the escape key is pressed', async () => {
    const user = setupUser();
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });

    // Press the colon key directly
    await user.keyboard(':');

    expect(screen.getByRole('dialog', { name: /command console/i })).toBeInTheDocument();
  });

  it('does not open command console when semicolon is pressed without shift', async () => {
    const user = setupUser();
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });

    await user.keyboard(';');

    expect(screen.queryByRole('dialog', { name: /command console/i })).not.toBeInTheDocument();
  });

  it('closes the command console when Escape key is pressed', async () => {
    const user = setupUser();
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });

    // Open the console
    await user.keyboard(':');
    expect(screen.getByRole('dialog', { name: /Command console/i })).toBeInTheDocument();

    // Close with escape
    await user.keyboard('{Escape}');

    // Wait for the console to be removed from the DOM (after animation)
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: /Command console/i })).not.toBeInTheDocument();
    });
  });

  it('does not close the command console with escape if it is already closed', async () => {
    const user = setupUser();
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });

    expect(screen.queryByRole('dialog', { name: /Command console/i })).not.toBeInTheDocument();

    // Press escape when console is closed - should do nothing
    await user.keyboard('{Escape}');

    // Console should still be closed
    expect(screen.queryByRole('dialog', { name: /Command console/i })).not.toBeInTheDocument();
  });
});
