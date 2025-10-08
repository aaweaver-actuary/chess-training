import { render, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { ReviewDashboard } from '../ReviewDashboard';
import type { ReviewOverview } from '../../services/ReviewPlanner';

describe('ReviewDashboard', () => {
  const overview: ReviewOverview = {
    progress: {
      totalDue: 10,
      completedToday: 6,
      remaining: 4,
      completionRate: 0.6,
      accuracyRate: 0.6,
    },
    tension: {
      backlogPressure: 'moderate',
      accuracyRisk: 'stable',
    },
    recommendation: {
      primaryAction: 'Finish the remaining four reviews',
      secondaryAction: "Skim yesterday's mistakes to keep context fresh",
    },
    upcomingUnlocks: [
      {
        id: 'unlock-1',
        move: 'd4',
        idea: 'Queens pawn space gain',
        scheduledFor: '2024-01-13',
      },
      {
        id: 'unlock-2',
        move: 'c4',
        idea: 'English transposition',
        scheduledFor: '2024-01-14',
      },
    ],
  };

  it('renders review metrics and recommendations', () => {
    render(<ReviewDashboard overview={overview} />);

    expect(screen.getByRole('heading', { name: /Daily Review Summary/i })).toBeInTheDocument();
    expect(screen.getByText('10')).toBeInTheDocument();
    expect(screen.getByText('6')).toBeInTheDocument();
    expect(screen.getByText('4')).toBeInTheDocument();
    expect(screen.getByText(/60% complete/i)).toBeInTheDocument();
    expect(screen.getByText(/Accuracy 60%/i)).toBeInTheDocument();
    expect(screen.getByText(overview.recommendation.primaryAction)).toBeInTheDocument();
    expect(screen.getByText(overview.recommendation.secondaryAction)).toBeInTheDocument();
  });

  it('lists upcoming unlocks with contextual descriptions', () => {
    render(<ReviewDashboard overview={overview} />);

    const list = screen.getByRole('list', { name: /upcoming unlocks/i });
    const items = within(list).getAllByRole('listitem');

    expect(items).toHaveLength(2);
    expect(items[0]).toHaveTextContent('d4');
    expect(items[0]).toHaveTextContent('Queens pawn space gain');
    expect(items[1]).toHaveTextContent('c4');
    expect(items[1]).toHaveTextContent('English transposition');
  });

  it('communicates cleared backlog and critical accuracy states', () => {
    render(
      <ReviewDashboard
        overview={{
          ...overview,
          tension: {
            backlogPressure: 'cleared',
            accuracyRisk: 'critical',
          },
          progress: {
            ...overview.progress,
            remaining: 0,
            completionRate: 1,
            accuracyRate: 0.72,
          },
        }}
      />,
    );

    expect(screen.getByText('Cleared')).toBeInTheDocument();
    expect(screen.getByText('Accuracy critical')).toBeInTheDocument();
  });
});
