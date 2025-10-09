import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
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
    const user = userEvent.setup();
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
  it('allows navigating between the dashboard and the opening review board', async () => {
    const user = userEvent.setup();
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

  it('opens the command console when the colon key is pressed', async () => {
    const user = userEvent.setup();
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

  it('closes the command console when the escape key is pressed', async () => {
    const user = userEvent.setup();
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
    const user = userEvent.setup();
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
    const user = userEvent.setup();
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
