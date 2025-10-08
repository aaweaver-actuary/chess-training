import type { FC } from 'react';
import { Link } from 'react-router-dom';

import { ReviewDashboard } from '../components/ReviewDashboard';
import type { ReviewOverview } from '../services/ReviewPlanner';

type DashboardPageProps = {
  overview: ReviewOverview;
  openingPath: string;
  canStartOpening: boolean;
};

const buildLinkClass = (enabled: boolean): string =>
  enabled ? 'nav-link floating-action' : 'nav-link floating-action nav-link-disabled';

export const DashboardPage: FC<DashboardPageProps> = ({
  overview,
  openingPath,
  canStartOpening,
}) => (
  <main className="app-shell dashboard-page">
    <ReviewDashboard overview={overview} />
    <nav aria-label="Review navigation" className="dashboard-navigation">
      <Link
        to={openingPath}
        className={buildLinkClass(canStartOpening)}
        aria-disabled={!canStartOpening}
        tabIndex={canStartOpening ? 0 : -1}
      >
        Start Opening Review
      </Link>
    </nav>
  </main>
);
