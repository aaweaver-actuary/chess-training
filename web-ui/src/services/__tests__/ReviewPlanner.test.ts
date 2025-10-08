import { describe, expect, it } from 'vitest';

import { ReviewPlanner, type ReviewSnapshot } from '../ReviewPlanner';

describe('ReviewPlanner', () => {
  const baseSnapshot: ReviewSnapshot = {
    dueCards: 20,
    completedCards: 5,
    accuracyRate: 0.6,
    streakLength: 3,
    upcomingUnlocks: [
      {
        id: 'unlock-1',
        move: 'e4',
        idea: 'King pawn opening control',
        scheduledFor: '2024-01-10',
      },
      {
        id: 'unlock-2',
        move: 'Nf3',
        idea: 'Attack e5 pawn',
        scheduledFor: '2024-01-12',
      },
    ],
  };

  it('summarizes progress metrics and backlog pressure', () => {
    const planner = new ReviewPlanner();

    const overview = planner.buildOverview(baseSnapshot);

    expect(overview.progress.totalDue).toBe(20);
    expect(overview.progress.completedToday).toBe(5);
    expect(overview.progress.remaining).toBe(15);
    expect(overview.progress.completionRate).toBeCloseTo(0.25);
    expect(overview.tension.backlogPressure).toBe('high');
    expect(overview.recommendation.primaryAction).toBe('Catch up on overdue reviews');
    expect(overview.recommendation.secondaryAction).toContain(
      'Reinforce accuracy with short tactics drills',
    );
  });

  it('detects a strong day and encourages expanding repertoire', () => {
    const planner = new ReviewPlanner();

    const overview = planner.buildOverview({
      ...baseSnapshot,
      dueCards: 6,
      completedCards: 6,
      accuracyRate: 0.92,
      streakLength: 12,
    });

    expect(overview.progress.remaining).toBe(0);
    expect(overview.tension.backlogPressure).toBe('cleared');
    expect(overview.recommendation.primaryAction).toBe('Add one new line to your repertoire');
    expect(overview.recommendation.secondaryAction).toBe(
      'Review high-value mistakes from the past week',
    );
  });

  it('suggests structured blocks when backlog is moderate', () => {
    const planner = new ReviewPlanner();

    const overview = planner.buildOverview({
      ...baseSnapshot,
      dueCards: 9,
      completedCards: 5,
      accuracyRate: 0.84,
      streakLength: 4,
    });

    expect(overview.tension.backlogPressure).toBe('moderate');
    expect(overview.tension.accuracyRisk).toBe('watch');
    expect(overview.recommendation.primaryAction).toBe(
      "Work through today's reviews in two focused blocks",
    );
    expect(overview.recommendation.secondaryAction).toBe(
      'Log any mistakes immediately to revisit tomorrow',
    );
  });

  it('stabilizes accuracy when backlog is low but precision is critical', () => {
    const planner = new ReviewPlanner();

    const overview = planner.buildOverview({
      ...baseSnapshot,
      dueCards: 4,
      completedCards: 3,
      accuracyRate: 0.68,
      streakLength: 2,
    });

    expect(overview.tension.backlogPressure).toBe('low');
    expect(overview.tension.accuracyRisk).toBe('critical');
    expect(overview.recommendation.primaryAction).toBe(
      'Stabilize accuracy with quick refresh drills',
    );
    expect(overview.recommendation.secondaryAction).toBe(
      'Tag the weakest lines for focused review',
    );
  });

  it('encourages a final focus block when accuracy is watch-listed', () => {
    const planner = new ReviewPlanner();

    const overview = planner.buildOverview({
      ...baseSnapshot,
      dueCards: 7,
      completedCards: 7,
      accuracyRate: 0.85,
      streakLength: 5,
    });

    expect(overview.tension.backlogPressure).toBe('cleared');
    expect(overview.tension.accuracyRisk).toBe('watch');
    expect(overview.recommendation.primaryAction).toBe(
      'Finish the day with one more focused review block',
    );
  });

  it('treats zero due cards as a fully completed day', () => {
    const planner = new ReviewPlanner();

    const overview = planner.buildOverview({
      ...baseSnapshot,
      dueCards: 0,
      completedCards: 0,
      accuracyRate: 1,
      streakLength: 1,
    });

    expect(overview.progress.completionRate).toBe(1);
    expect(overview.tension.backlogPressure).toBe('cleared');
  });

  it('keeps momentum suggestions when backlog is low and accuracy is steady', () => {
    const planner = new ReviewPlanner();

    const overview = planner.buildOverview({
      ...baseSnapshot,
      dueCards: 3,
      completedCards: 2,
      accuracyRate: 0.9,
      streakLength: 3,
    });

    expect(overview.tension.backlogPressure).toBe('low');
    expect(overview.recommendation.primaryAction).toBe(
      'Complete the remaining reviews in a single sprint',
    );
  });

  it('prioritizes rebuilding confidence when accuracy collapses despite cleared backlog', () => {
    const planner = new ReviewPlanner();

    const overview = planner.buildOverview({
      ...baseSnapshot,
      dueCards: 4,
      completedCards: 4,
      accuracyRate: 0.7,
      streakLength: 2,
    });

    expect(overview.tension.backlogPressure).toBe('cleared');
    expect(overview.tension.accuracyRisk).toBe('critical');
    expect(overview.recommendation.primaryAction).toBe(
      'Rebuild confidence on the weakest variations',
    );
  });

  it('plans for tomorrow when the backlog is clear and accuracy is steady', () => {
    const planner = new ReviewPlanner();

    const overview = planner.buildOverview({
      ...baseSnapshot,
      dueCards: 5,
      completedCards: 5,
      accuracyRate: 0.95,
      streakLength: 4,
    });

    expect(overview.tension.backlogPressure).toBe('cleared');
    expect(overview.tension.accuracyRisk).toBe('stable');
    expect(overview.recommendation.primaryAction).toBe(
      "Plan tomorrow's unlock and keep the momentum",
    );
  });

  it('raises when provided with invalid snapshot data', () => {
    const planner = new ReviewPlanner();

    expect(() =>
      planner.buildOverview({
        ...baseSnapshot,
        dueCards: -1,
      }),
    ).toThrowError('Review counts cannot be negative');

    expect(() =>
      planner.buildOverview({
        ...baseSnapshot,
        accuracyRate: 1.2,
      }),
    ).toThrowError('Accuracy must be between 0 and 1');
  });
});
