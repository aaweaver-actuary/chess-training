import type { FC } from 'react';
import { Link } from 'react-router-dom';

import { PgnImportPane } from '../components/PgnImportPane';
import { ReviewDashboard } from '../components/ReviewDashboard';
import type { ReviewOverview } from '../services/ReviewPlanner';
import type { DetectedOpeningLine, ImportResult } from '../types/repertoire';

type DashboardPageProps = {
  overview: ReviewOverview;
  openingPath: string;
  canStartOpening: boolean;
  onImportLine: (line: DetectedOpeningLine) => ImportResult;
};

const buildLinkClass = (enabled: boolean): string =>
  enabled ? 'nav-link floating-action' : 'nav-link floating-action nav-link-disabled';

export const DashboardPage: FC<DashboardPageProps> = ({
  overview,
  openingPath,
  canStartOpening,
  onImportLine,
}) => (
  <main className="app-shell dashboard-page">
    <PgnImportPane onImportLine={onImportLine} />
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
