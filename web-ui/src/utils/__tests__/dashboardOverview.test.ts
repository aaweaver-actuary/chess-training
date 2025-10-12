import { describe, expect, it } from 'vitest';

import type { ReviewOverview } from '../../types/reviewOverview';
import type { SessionStats } from '../../types/gateway';
import type { ScheduledOpeningLine } from '../../types/repertoire';

import { composeOverview, extendOverviewWithImports } from '../dashboardOverview';

describe('dashboardOverview', () => {
  const baselineOverview: ReviewOverview = {
    progress: {
      totalDue: 18,
      remaining: 9,
      completedToday: 9,
      completionRate: 0.5,
      accuracyRate: 0.8,
    },
    tension: {
      backlogPressure: 'low',
      accuracyRisk: 'stable',
    },
    recommendation: {
      primaryAction: 'Keep momentum',
      secondaryAction: 'Focus on accuracy',
    },
    upcomingUnlocks: [],
  };

  it('falls back to the baseline when stats are undefined', () => {
    const overview = composeOverview(baselineOverview, undefined);
    expect(overview).toEqual(baselineOverview);
  });

  it('derives remaining count and completion rate from stats', () => {
    const stats: SessionStats = {
      reviews_today: 4,
      accuracy: 0.5,
      avg_latency_ms: 1234,
      due_count: 6,
      completed_count: 4,
    };

    const overview = composeOverview(baselineOverview, stats);

    expect(overview.progress).toEqual({
      totalDue: 6,
      completedToday: 4,
      remaining: 2,
      completionRate: 2 / 3,
      accuracyRate: 0.5,
    });
  });

  it('treats a zero due count as complete', () => {
    const stats: SessionStats = {
      reviews_today: 0,
      accuracy: 0.9,
      avg_latency_ms: 1000,
      due_count: 0,
      completed_count: 0,
    };

    const overview = composeOverview(baselineOverview, stats);

    expect(overview.progress).toMatchObject({
      totalDue: 0,
      completedToday: 0,
      remaining: 0,
      completionRate: 1,
      accuracyRate: 0.9,
    });
  });

  it('appends imported lines to the upcoming unlocks', () => {
    const overview = composeOverview(baselineOverview, undefined);
    const imports: ScheduledOpeningLine[] = [
      {
        id: 'import-1',
        opening: 'Italian Game',
        color: 'White',
        moves: ['e4', 'e5', 'Nf3'],
        display: '1.e4 e5 2.Nf3',
        scheduledFor: '2024-04-01',
      },
    ];

    const augmented = extendOverviewWithImports(overview, imports);

    expect(augmented.upcomingUnlocks).toEqual([
      {
        id: 'import-1',
        move: 'Italian Game (White)',
        idea: 'Line: 1.e4 e5 2.Nf3',
        scheduledFor: '2024-04-01',
      },
    ]);
  });
});
