export type BacklogPressure = 'cleared' | 'low' | 'moderate' | 'high';

export type AccuracyRisk = 'stable' | 'watch' | 'critical';

export type Recommendation = {
  primaryAction: string;
  secondaryAction: string;
};

export type UpcomingUnlock = {
  id: string;
  move: string;
  idea: string;
  scheduledFor: string;
};

export type ProgressOverview = {
  totalDue: number;
  completedToday: number;
  remaining: number;
  completionRate: number;
  accuracyRate: number;
};

export type TensionOverview = {
  backlogPressure: BacklogPressure;
  accuracyRisk: AccuracyRisk;
};

export type ReviewOverview = {
  progress: ProgressOverview;
  tension: TensionOverview;
  recommendation: Recommendation;
  upcomingUnlocks: UpcomingUnlock[];
};
