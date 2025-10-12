import type { ReviewOverview } from '../types/reviewOverview';

export const baselineReviewOverview: ReviewOverview = {
  progress: {
    totalDue: 18,
    completedToday: 11,
    remaining: 7,
    completionRate: 11 / 18,
    accuracyRate: 0.86,
  },
  tension: {
    backlogPressure: 'moderate',
    accuracyRisk: 'watch',
  },
  recommendation: {
    primaryAction: "Work through today's reviews in two focused blocks",
    secondaryAction: 'Log any mistakes immediately to revisit tomorrow',
  },
  upcomingUnlocks: [
    {
      id: 'unlock-italian',
      move: 'Bc4',
      idea: 'Pressure f7 in the Italian Game',
      scheduledFor: '2024-01-14',
    },
    {
      id: 'unlock-scandi',
      move: 'Qxd5',
      idea: 'Centralize the queen in the Scandinavian',
      scheduledFor: '2024-01-15',
    },
    {
      id: 'unlock-tactic',
      move: 'Nxf7+',
      idea: 'Tactic: Greek gift sacrifice',
      scheduledFor: '2024-01-16',
    },
  ],
};
