import type { SessionStats } from '../../types/gateway';
import type { ImportResult, ScheduledOpeningLine } from '../../types/repertoire';

export type DashboardMetric = {
  id: string;
  label: string;
  value: string;
  helperText?: string;
  tone?: 'default' | 'success' | 'warning' | 'danger';
};

export type DashboardBadge = {
  id: string;
  tone: 'success' | 'info' | 'warning' | 'danger';
  title: string;
  description?: string;
};

export type UpcomingUnlock = {
  line: ScheduledOpeningLine;
  friendlyDate: string;
};

export type DashboardOverview = {
  metrics: DashboardMetric[];
  badge: DashboardBadge | null;
  upcomingUnlocks: UpcomingUnlock[];
  importResults: ImportResult[];
  stats: SessionStats | null;
};

export type DashboardViewModel = {
  load: () => Promise<DashboardOverview>;
  refresh: () => Promise<DashboardOverview>;
  subscribe: (listener: (overview: DashboardOverview) => void) => () => void;
  applyImportResults: (results: ImportResult[]) => void;
  updateSessionStats: (stats: SessionStats) => void;
};
