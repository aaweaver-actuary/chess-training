import type { SessionStats } from '../../types/gateway';
import type { ImportResult, ScheduledOpeningLine } from '../../types/repertoire';
import { formatUnlockDate as defaultFormatUnlockDate } from '../../utils/formatUnlockDate';
import type {
  ReviewOverview,
  ReviewSnapshot,
} from '../../services/ReviewPlanner';
import { ReviewPlanner } from '../../services/ReviewPlanner';

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
  id: string;
  title: string;
  description: string;
  scheduledFor: string;
  friendlyDate: string;
  source: 'baseline' | 'imported';
  line?: ScheduledOpeningLine;
};

export type DashboardOverview = {
  metrics: DashboardMetric[];
  badge: DashboardBadge | null;
  accuracyBadge: DashboardBadge | null;
  upcomingUnlocks: UpcomingUnlock[];
  importResults: ImportResult[];
  stats: SessionStats | null;
  recommendation: ReviewOverview['recommendation'];
};

export interface DashboardViewModel {
  load(): Promise<DashboardOverview>;
  refresh(): Promise<DashboardOverview>;
  getCurrentOverview(): DashboardOverview;
  subscribe(listener: (overview: DashboardOverview) => void): () => void;
  applyImportResults(results: ImportResult[]): void;
  updateSessionStats(stats: SessionStats | null): void;
}

export type DashboardViewModelDependencies = {
  reviewPlanner: ReviewPlanner;
  baselineSnapshot: ReviewSnapshot;
  formatUnlockDate?: (input: string) => string;
};

type Listener = (overview: DashboardOverview) => void;

const BACKLOG_BADGE_MAP = {
  cleared: {
    tone: 'success',
    title: 'Cleared',
    description: 'Queue fully cleared — add a new line today',
  },
  low: {
    tone: 'success',
    title: 'Low backlog',
    description: 'Finish the remaining reviews in a single sprint',
  },
  moderate: {
    tone: 'warning',
    title: 'Moderate backlog',
    description: 'Work through reviews in two focused blocks',
  },
  high: {
    tone: 'danger',
    title: 'High backlog',
    description: 'Catch up on overdue reviews as the top priority',
  },
} as const;

const ACCURACY_BADGE_MAP = {
  stable: {
    tone: 'success',
    title: 'Accuracy stable',
    description: 'Keep reinforcing strengths with light review',
  },
  watch: {
    tone: 'warning',
    title: 'Accuracy watch',
    description: 'Reinforce recent mistakes to steady accuracy',
  },
  critical: {
    tone: 'danger',
    title: 'Accuracy critical',
    description: 'Rebuild confidence on the weakest variations',
  },
} as const;

const COMPLETION_TONE_MAP = {
  stable: 'success',
  watch: 'warning',
  critical: 'danger',
} as const satisfies Record<keyof typeof ACCURACY_BADGE_MAP, DashboardMetric['tone']>;

export class DefaultDashboardViewModel implements DashboardViewModel {
  private readonly reviewPlanner: ReviewPlanner;
  private readonly baselineSnapshot: ReviewSnapshot;
  private readonly formatUnlockDate: (input: string) => string;

  private stats: SessionStats | null = null;
  private importResults: ImportResult[] = [];
  private readonly plannedLines = new Map<string, ScheduledOpeningLine>();
  private readonly listeners = new Set<Listener>();
  private currentOverview: DashboardOverview | null = null;

  constructor({
    reviewPlanner,
    baselineSnapshot,
    formatUnlockDate = defaultFormatUnlockDate,
  }: DashboardViewModelDependencies) {
    this.reviewPlanner = reviewPlanner;
    this.baselineSnapshot = { ...baselineSnapshot };
    this.formatUnlockDate = formatUnlockDate;
    this.currentOverview = this.composeOverview();
  }

  public async load(): Promise<DashboardOverview> {
    if (!this.currentOverview) {
      this.currentOverview = this.composeOverview();
    }
    return this.currentOverview;
  }

  public async refresh(): Promise<DashboardOverview> {
    this.currentOverview = this.composeOverview();
    return this.currentOverview;
  }

  public getCurrentOverview(): DashboardOverview {
    if (!this.currentOverview) {
      this.currentOverview = this.composeOverview();
    }
    return this.currentOverview;
  }

  public subscribe(listener: Listener): () => void {
    this.listeners.add(listener);
    listener(this.getCurrentOverview());
    return () => {
      this.listeners.delete(listener);
    };
  }

  public applyImportResults(results: ImportResult[]): void {
    if (results.length === 0) {
      return;
    }

    this.importResults = [...this.importResults, ...results];
    results.forEach((result) => {
      this.plannedLines.set(result.line.id, result.line);
    });

    this.emit();
  }

  public updateSessionStats(stats: SessionStats | null): void {
    this.stats = stats;
    this.emit();
  }

  private emit(): void {
    const overview = this.composeOverview();
    this.currentOverview = overview;
    this.listeners.forEach((listener) => listener(overview));
  }

  private composeOverview(): DashboardOverview {
    const snapshot = this.composeSnapshot();
    const reviewOverview = this.reviewPlanner.buildOverview(snapshot);

    const metrics = this.buildMetrics(reviewOverview);
    const badge = this.buildBacklogBadge(reviewOverview);
    const accuracyBadge = this.buildAccuracyBadge(reviewOverview);
    const upcomingUnlocks = this.buildUpcomingUnlocks(reviewOverview);

    return {
      metrics,
      badge,
      accuracyBadge,
      upcomingUnlocks,
      importResults: this.importResults,
      stats: this.stats,
      recommendation: reviewOverview.recommendation,
    };
  }

  private composeSnapshot(): ReviewSnapshot {
    const upcomingUnlocks = [
      ...this.baselineSnapshot.upcomingUnlocks,
      ...Array.from(this.plannedLines.values()).map((line) => ({
        id: line.id,
        move: `${line.opening} (${line.color})`,
        idea: `Line: ${line.display}`,
        scheduledFor: line.scheduledFor,
      })),
    ];

    if (!this.stats) {
      return {
        ...this.baselineSnapshot,
        upcomingUnlocks,
      };
    }

    return {
      dueCards: this.stats.due_count,
      completedCards: this.stats.completed_count,
      accuracyRate: this.stats.accuracy,
      streakLength: this.baselineSnapshot.streakLength,
      upcomingUnlocks,
    };
  }

  private buildMetrics(overview: ReviewOverview): DashboardMetric[] {
    const {
      totalDue,
      completedToday,
      remaining,
      completionRate,
      accuracyRate,
    } = overview.progress;
    const accuracyBadge = overview.tension.accuracyRisk;

    const completionPercentage = Math.round(completionRate * 100);
    const accuracyPercentage = Math.round(accuracyRate * 100);

    return [
      {
        id: 'due-today',
        label: 'Due Today',
        value: String(totalDue),
        tone: 'default',
      },
      {
        id: 'completed-today',
        label: 'Completed',
        value: String(completedToday),
        tone: 'success',
      },
      {
        id: 'remaining',
        label: 'Remaining',
        value: String(remaining),
        tone: remaining === 0 ? 'success' : 'warning',
      },
      {
        id: 'completion-rate',
        label: 'Completion',
        value: `${completionPercentage}% complete`,
        helperText: `Accuracy ${accuracyPercentage}%`,
        tone: COMPLETION_TONE_MAP[accuracyBadge],
      },
    ];
  }

  private buildBacklogBadge(overview: ReviewOverview): DashboardBadge {
    const tension = overview.tension.backlogPressure;
    const definition = BACKLOG_BADGE_MAP[tension];

    return {
      id: 'backlog-status',
      tone: definition.tone,
      title: definition.title,
      description: definition.description,
    };
  }

  private buildAccuracyBadge(overview: ReviewOverview): DashboardBadge {
    const risk = overview.tension.accuracyRisk;
    const definition = ACCURACY_BADGE_MAP[risk];

    return {
      id: 'accuracy-status',
      tone: definition.tone,
      title: definition.title,
      description: definition.description,
    };
  }

  private buildUpcomingUnlocks(overview: ReviewOverview): UpcomingUnlock[] {
    return overview.upcomingUnlocks.map((unlock) => {
      const plannedLine = this.plannedLines.get(unlock.id);

      if (plannedLine) {
        return {
          id: plannedLine.id,
          title: `${plannedLine.opening} (${plannedLine.color})`,
          description: `Line: ${plannedLine.display}`,
          scheduledFor: plannedLine.scheduledFor,
          friendlyDate: this.formatUnlockDate(plannedLine.scheduledFor),
          source: 'imported' as const,
          line: plannedLine,
        };
      }

      return {
        id: unlock.id,
        title: unlock.move,
        description: unlock.idea,
        scheduledFor: unlock.scheduledFor,
        friendlyDate: this.formatUnlockDate(unlock.scheduledFor),
        source: 'baseline' as const,
      };
    });
  }
}
