import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

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
      position_fen: 'start',
      prompt: 'Play the move',
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
});

describe('App', () => {
  it('starts the session on mount and renders live stats', async () => {
    render(<App />);

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });
    expect(screen.getByRole('heading', { name: /Daily Review Summary/i })).toBeInTheDocument();
    expect(screen.getByText('12')).toBeInTheDocument();
    expect(screen.getByText('9')).toBeInTheDocument();
  });

  it('submits a grade when clicking a grade button', async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(screen.getByRole('button', { name: /Good/i }));

    expect(mockedStore.getState().submitGrade).toHaveBeenCalledWith('Good', expect.any(Number));
  });

  it('falls back to the baseline overview when stats are unavailable', async () => {
    mockedStore.getState().stats = undefined;
    render(<App />);

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
    render(<App />);

    await waitFor(() => {
      expect(mockedStore.getState().start).toHaveBeenCalled();
    });
    const dueCard = screen.getByText('Due Today').closest('.metric-card');
    expect(dueCard).not.toBeNull();
    expect(within(dueCard as HTMLElement).getByText('0')).toBeInTheDocument();
    expect(screen.getByText(/100% complete/i)).toBeInTheDocument();
  });
});
