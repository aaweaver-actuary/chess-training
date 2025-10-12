import type { ReviewOverview } from '../types/reviewOverview';
import type { SessionStats } from '../types/gateway';
import type { ScheduledOpeningLine } from '../types/repertoire';

const deriveProgress = (
  baseline: ReviewOverview['progress'],
  stats?: SessionStats,
): ReviewOverview['progress'] => {
  if (!stats) {
    return baseline;
  }

  const totalDue = stats.due_count;
  const completed = stats.completed_count;
  const remaining = Math.max(totalDue - completed, 0);
  const completionRate = totalDue === 0 ? 1 : completed / totalDue;

  return {
    ...baseline,
    totalDue,
    completedToday: completed,
    remaining,
    completionRate,
    accuracyRate: stats.accuracy,
  };
};

export const composeOverview = (
  baseline: ReviewOverview,
  stats: SessionStats | undefined,
): ReviewOverview => ({
  ...baseline,
  progress: deriveProgress(baseline.progress, stats),
});

export const extendOverviewWithImports = (
  overview: ReviewOverview,
  lines: ScheduledOpeningLine[],
): ReviewOverview => {
  if (lines.length === 0) {
    return overview;
  }

  return {
    ...overview,
    upcomingUnlocks: [
      ...overview.upcomingUnlocks,
      ...lines.map((line) => ({
        id: line.id,
        move: `${line.opening} (${line.color})`,
        idea: `Line: ${line.display}`,
        scheduledFor: line.scheduledFor,
      })),
    ],
  };
};
