import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { MemoryRouter } from 'react-router-dom';

import { DashboardPage } from '../DashboardPage';
import type { ReviewOverview } from '../../services/ReviewPlanner';

const buildOverview = (overrides: Partial<ReviewOverview> = {}): ReviewOverview => ({
  progress: {
    totalDue: 12,
    completedToday: 9,
    remaining: 3,
    completionRate: 0.75,
    accuracyRate: 0.8,
    ...(overrides.progress ?? {}),
  },
  tension: {
    backlogPressure: 'moderate',
    accuracyRisk: 'watch',
    ...(overrides.tension ?? {}),
  },
  recommendation: {
    primaryAction: 'Keep the streak alive',
    secondaryAction: 'Log any mistakes to revisit tomorrow',
    ...(overrides.recommendation ?? {}),
  },
  upcomingUnlocks: overrides.upcomingUnlocks ?? [
    {
      id: 'unlock-1',
      move: 'e4',
      idea: 'Control the center',
      scheduledFor: '2024-01-01T00:00:00Z',
    },
  ],
});

describe('DashboardPage', () => {
  it('enables navigation when an opening review can start', () => {
    render(
      <MemoryRouter>
        <DashboardPage overview={buildOverview()} openingPath="/review/opening" canStartOpening />
      </MemoryRouter>,
    );

    const link = screen.getByRole('link', { name: /Start opening review/i });
    expect(link).toHaveAttribute('href', '/review/opening');
    expect(link).toHaveClass('nav-link');
    expect(link).not.toHaveClass('nav-link-disabled');
    expect(link).toHaveAttribute('aria-disabled', 'false');
    expect(link).toHaveAttribute('tabindex', '0');
  });

  it('disables navigation when no opening review is available', () => {
    render(
      <MemoryRouter>
        <DashboardPage
          overview={buildOverview()}
          openingPath="/review/opening"
          canStartOpening={false}
        />
      </MemoryRouter>,
    );

    const link = screen.getByRole('link', { name: /Start opening review/i });
    expect(link).toHaveClass('nav-link-disabled');
    expect(link).toHaveAttribute('aria-disabled', 'true');
    expect(link).toHaveAttribute('tabindex', '-1');
  });
});
