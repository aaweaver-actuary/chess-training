import { describe, expect, it, vi, beforeEach } from 'vitest';

import { DefaultDashboardViewModel } from '../DashboardViewModel.js';
import { ReviewPlanner } from '../../../services/ReviewPlanner.js';
import { sampleSnapshot } from '../../../fixtures/sampleSnapshot.js';
import type { SessionStats } from '../../../types/gateway.js';
import type { ImportResult, ScheduledOpeningLine } from '../../../types/repertoire.js';

const formatUnlockDate = vi.fn((input: string) => `friendly-${input}`);

const createViewModel = () =>
  new DefaultDashboardViewModel({
    reviewPlanner: new ReviewPlanner(),
    baselineSnapshot: sampleSnapshot,
    formatUnlockDate,
  });

describe('DefaultDashboardViewModel', () => {
  beforeEach(() => {
    formatUnlockDate.mockClear();
  });

  it('produces baseline metrics, badges, and unlock strings on load', async () => {
    const viewModel = createViewModel();

    const overview = await viewModel.load();

    expect(overview.metrics).toEqual([
      { id: 'due-today', label: 'Due Today', value: '18', tone: 'default' },
      { id: 'completed-today', label: 'Completed', value: '11', tone: 'success' },
      { id: 'remaining', label: 'Remaining', value: '7', tone: 'warning' },
      {
        id: 'completion-rate',
        label: 'Completion',
        value: '61% complete',
        helperText: 'Accuracy 86%',
        tone: 'warning',
      },
    ]);
    expect(overview.badge).toEqual({
      id: 'backlog-status',
      tone: 'warning',
      title: 'Moderate backlog',
      description: 'Work through reviews in two focused blocks',
    });
    expect(overview.accuracyBadge).toEqual({
      id: 'accuracy-status',
      tone: 'warning',
      title: 'Accuracy watch',
      description: 'Reinforce recent mistakes to steady accuracy',
    });
    expect(overview.recommendation).toEqual({
      primaryAction: "Work through today's reviews in two focused blocks",
      secondaryAction: 'Log any mistakes immediately to revisit tomorrow',
    });
    expect(overview.upcomingUnlocks).toEqual([
      {
        id: 'unlock-italian',
        title: 'Bc4',
        description: 'Pressure f7 in the Italian Game',
        scheduledFor: '2024-01-14',
        friendlyDate: 'friendly-2024-01-14',
        source: 'baseline',
      },
      {
        id: 'unlock-scandi',
        title: 'Qxd5',
        description: 'Centralize the queen in the Scandinavian',
        scheduledFor: '2024-01-15',
        friendlyDate: 'friendly-2024-01-15',
        source: 'baseline',
      },
      {
        id: 'unlock-tactic',
        title: 'Nxf7+',
        description: 'Tactic: Greek gift sacrifice',
        scheduledFor: '2024-01-16',
        friendlyDate: 'friendly-2024-01-16',
        source: 'baseline',
      },
    ]);
  });

  it('updates progress metrics and badges when session stats change', async () => {
    const viewModel = createViewModel();
    await viewModel.load();

    const listener = vi.fn();
    viewModel.subscribe(listener);

    expect(listener).toHaveBeenCalledTimes(1);

    const stats: SessionStats = {
      reviews_today: 9,
      accuracy: 0.72,
      avg_latency_ms: 1800,
      due_count: 12,
      completed_count: 9,
    };

    viewModel.updateSessionStats(stats);

    expect(listener).toHaveBeenCalledTimes(2);
    const updatedOverview = listener.mock.calls[1][0];

    expect(updatedOverview.metrics).toEqual([
      { id: 'due-today', label: 'Due Today', value: '12', tone: 'default' },
      { id: 'completed-today', label: 'Completed', value: '9', tone: 'success' },
      { id: 'remaining', label: 'Remaining', value: '3', tone: 'warning' },
      {
        id: 'completion-rate',
        label: 'Completion',
        value: '75% complete',
        helperText: 'Accuracy 72%',
        tone: 'danger',
      },
    ]);
    expect(updatedOverview.badge).toEqual({
      id: 'backlog-status',
      tone: 'success',
      title: 'Low backlog',
      description: 'Finish the remaining reviews in a single sprint',
    });
    expect(updatedOverview.accuracyBadge).toEqual({
      id: 'accuracy-status',
      tone: 'danger',
      title: 'Accuracy critical',
      description: 'Rebuild confidence on the weakest variations',
    });
    expect(updatedOverview.stats).toEqual(stats);

    const refreshed = await viewModel.refresh();
    expect(refreshed).toEqual(updatedOverview);
  });

  it('treats zero due counts as fully complete', async () => {
    const viewModel = createViewModel();
    await viewModel.load();

    viewModel.updateSessionStats({
      reviews_today: 0,
      accuracy: 0.82,
      avg_latency_ms: 0,
      due_count: 0,
      completed_count: 0,
    });

    const overview = await viewModel.refresh();
    expect(overview.metrics).toEqual([
      { id: 'due-today', label: 'Due Today', value: '0', tone: 'default' },
      { id: 'completed-today', label: 'Completed', value: '0', tone: 'success' },
      { id: 'remaining', label: 'Remaining', value: '0', tone: 'success' },
      {
        id: 'completion-rate',
        label: 'Completion',
        value: '100% complete',
        helperText: 'Accuracy 82%',
        tone: 'warning',
      },
    ]);
  });

  it('appends import results to upcoming unlocks with friendly dates', async () => {
    const viewModel = createViewModel();
    await viewModel.load();

    const newLine: ScheduledOpeningLine = {
      id: 'line-sicilian',
      opening: 'Sicilian Defence',
      color: 'Black',
      moves: ['e4', 'c5', 'Nf3', 'd6'],
      display: '1.e4 c5 2.Nf3 d6',
      scheduledFor: '2024-02-01',
    };
    const results: ImportResult[] = [{ added: true, line: newLine }];

    const notifications: unknown[] = [];
    const unsubscribe = viewModel.subscribe((overview) => {
      notifications.push(overview);
    });

    viewModel.applyImportResults(results);

    const refreshed = await viewModel.refresh();

    expect(refreshed.importResults).toEqual(results);
    const appendedUnlock = refreshed.upcomingUnlocks.find((unlock) => unlock.id === newLine.id);
    expect(appendedUnlock).toEqual({
      id: 'line-sicilian',
      title: 'Sicilian Defence (Black)',
      description: 'Line: 1.e4 c5 2.Nf3 d6',
      scheduledFor: '2024-02-01',
      friendlyDate: 'friendly-2024-02-01',
      source: 'imported',
      line: newLine,
    });
    expect(formatUnlockDate).toHaveBeenCalledWith('2024-02-01');
    expect(notifications).toHaveLength(2);

    unsubscribe();
    viewModel.applyImportResults(results);
    expect(notifications).toHaveLength(2);
  });

  it('invokes listeners immediately with the latest overview on subscribe', () => {
    const viewModel = createViewModel();

    const listener = vi.fn();
    const unsubscribe = viewModel.subscribe(listener);

    expect(listener).toHaveBeenCalledTimes(1);
    expect(listener.mock.calls[0][0]).toEqual(viewModel.getCurrentOverview());

    unsubscribe();
    viewModel.applyImportResults([
      {
        added: true,
        line: {
          id: 'line-queued',
          opening: 'French Defence',
          color: 'Black',
          moves: ['e4', 'e6', 'd4', 'd5'],
          display: '1.e4 e6 2.d4 d5',
          scheduledFor: '2024-02-10',
        },
      },
    ]);

    expect(listener).toHaveBeenCalledTimes(1);
  });
});
