import type { FC } from 'react';

import type { ReviewOverview } from '../services/ReviewPlanner';

type MetricCardProps = {
  title: string;
  value: string;
  description?: string;
};

const MetricCard: FC<MetricCardProps> = ({ title, value, description }) => (
  <div className="metric-card">
    <p className="metric-title">{title}</p>
    <p className="metric-value">{value}</p>
    {description ? <p className="metric-description">{description}</p> : null}
  </div>
);

export type ReviewDashboardProps = {
  overview: ReviewOverview;
};

export const ReviewDashboard: FC<ReviewDashboardProps> = ({ overview }) => {
  const completionPercent = Math.round(overview.progress.completionRate * 100);
  const accuracyPercent = Math.round(overview.progress.accuracyRate * 100);
  const completionLabel = `${String(completionPercent)}% complete`;
  const accuracyLabel = `Accuracy ${String(accuracyPercent)}%`;

  return (
    <section className="dashboard" aria-labelledby="dashboard-heading">
      <header className="dashboard-header">
        <div>
          <p className="dashboard-subtitle">Chess Trainer</p>
          <h1 id="dashboard-heading">Daily Review Summary</h1>
        </div>
        <span className={`badge badge-${overview.tension.backlogPressure}`}>
          {overview.tension.backlogPressure === 'cleared'
            ? 'Cleared'
            : `${overview.tension.backlogPressure} backlog`}
        </span>
      </header>

      <div className="metrics-grid">
        <MetricCard title="Due Today" value={String(overview.progress.totalDue)} />
        <MetricCard title="Completed" value={String(overview.progress.completedToday)} />
        <MetricCard title="Remaining" value={String(overview.progress.remaining)} />
        <MetricCard title="Completion" value={completionLabel} description={accuracyLabel} />
      </div>

      <section className="recommendation" aria-labelledby="recommendation-heading">
        <h2 id="recommendation-heading">Today&apos;s Focus</h2>
        <p className="primary-action">{overview.recommendation.primaryAction}</p>
        <p className="secondary-action">{overview.recommendation.secondaryAction}</p>
      </section>

      <section className="upcoming" aria-labelledby="upcoming-heading">
        <div className="section-heading">
          <h2 id="upcoming-heading">Upcoming Unlocks</h2>
          <span className={`badge badge-${overview.tension.accuracyRisk}`}>
            {overview.tension.accuracyRisk === 'stable'
              ? 'Accuracy stable'
              : overview.tension.accuracyRisk === 'watch'
                ? 'Accuracy watch'
                : 'Accuracy critical'}
          </span>
        </div>
        <ul aria-label="upcoming unlocks">
          {overview.upcomingUnlocks.map((unlock) => (
            <li key={unlock.id} className="unlock-item">
              <div>
                <p className="unlock-move">{unlock.move}</p>
                <p className="unlock-idea">{unlock.idea}</p>
              </div>
              <time dateTime={unlock.scheduledFor} className="unlock-date">
                {new Date(unlock.scheduledFor).toLocaleDateString(undefined, {
                  month: 'short',
                  day: 'numeric',
                })}
              </time>
            </li>
          ))}
        </ul>
      </section>
    </section>
  );
};
