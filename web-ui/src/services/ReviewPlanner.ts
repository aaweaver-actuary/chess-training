export type UpcomingUnlock = {
  id: string;
  move: string;
  idea: string;
  scheduledFor: string;
};

export type ReviewSnapshot = {
  dueCards: number;
  completedCards: number;
  accuracyRate: number;
  streakLength: number;
  upcomingUnlocks: UpcomingUnlock[];
};

export type BacklogPressure = 'cleared' | 'low' | 'moderate' | 'high';
export type AccuracyRisk = 'stable' | 'watch' | 'critical';

export type ReviewOverview = {
  progress: {
    totalDue: number;
    completedToday: number;
    remaining: number;
    completionRate: number;
    accuracyRate: number;
  };
  tension: {
    backlogPressure: BacklogPressure;
    accuracyRisk: AccuracyRisk;
  };
  recommendation: {
    primaryAction: string;
    secondaryAction: string;
  };
  upcomingUnlocks: UpcomingUnlock[];
};

type Recommendation = ReviewOverview['recommendation'];

type RecommendationContext = {
  backlog: BacklogPressure;
  accuracy: AccuracyRisk;
  streakLength: number;
  remaining: number;
};

type RecommendationRule = {
  backlog?: BacklogPressure;
  accuracy?: AccuracyRisk;
  predicate?: (context: RecommendationContext) => boolean;
  recommendation: Recommendation;
};

const RECOMMENDATION_RULES: RecommendationRule[] = [
  {
    backlog: 'high',
    recommendation: {
      primaryAction: 'Catch up on overdue reviews',
      secondaryAction: 'Reinforce accuracy with short tactics drills',
    },
  },
  {
    backlog: 'moderate',
    recommendation: {
      primaryAction: "Work through today's reviews in two focused blocks",
      secondaryAction: 'Log any mistakes immediately to revisit tomorrow',
    },
  },
  {
    backlog: 'low',
    accuracy: 'critical',
    recommendation: {
      primaryAction: 'Stabilize accuracy with quick refresh drills',
      secondaryAction: 'Tag the weakest lines for focused review',
    },
  },
  {
    backlog: 'low',
    recommendation: {
      primaryAction: 'Complete the remaining reviews in a single sprint',
      secondaryAction: "Do a light skim of yesterday's problem areas",
    },
  },
  {
    accuracy: 'critical',
    recommendation: {
      primaryAction: 'Rebuild confidence on the weakest variations',
      secondaryAction: 'Schedule a tactics-only session for reinforcement',
    },
  },
  {
    accuracy: 'watch',
    recommendation: {
      primaryAction: 'Finish the day with one more focused review block',
      secondaryAction: 'Revisit the last set of inaccuracies to lock them in',
    },
  },
  {
    predicate: (context) => context.streakLength >= 10 && context.remaining === 0,
    recommendation: {
      primaryAction: 'Add one new line to your repertoire',
      secondaryAction: 'Review high-value mistakes from the past week',
    },
  },
];

const RECOMMENDATION_FALLBACK: Recommendation = {
  primaryAction: "Plan tomorrow's unlock and keep the momentum",
  secondaryAction: "Share today's success in your training journal",
};

export class ReviewPlanner {
  public buildOverview(snapshot: ReviewSnapshot): ReviewOverview {
    this.assertSnapshot(snapshot);

    const remaining = Math.max(snapshot.dueCards - snapshot.completedCards, 0);
    const completionRate =
      snapshot.dueCards === 0 ? 1 : snapshot.completedCards / snapshot.dueCards;

    const backlogPressure = this.assessBacklog(remaining);
    const accuracyRisk = this.assessAccuracy(snapshot.accuracyRate);
    const recommendation = this.deriveRecommendation({
      backlog: backlogPressure,
      accuracy: accuracyRisk,
      streakLength: snapshot.streakLength,
      remaining,
    });

    return {
      progress: {
        totalDue: snapshot.dueCards,
        completedToday: Math.min(snapshot.completedCards, snapshot.dueCards),
        remaining,
        completionRate,
        accuracyRate: snapshot.accuracyRate,
      },
      tension: {
        backlogPressure,
        accuracyRisk,
      },
      recommendation,
      upcomingUnlocks: [...snapshot.upcomingUnlocks],
    };
  }

  private assertSnapshot(snapshot: ReviewSnapshot): void {
    if (snapshot.dueCards < 0 || snapshot.completedCards < 0) {
      throw new Error('Review counts cannot be negative');
    }

    if (snapshot.accuracyRate < 0 || snapshot.accuracyRate > 1) {
      throw new Error('Accuracy must be between 0 and 1');
    }
  }

  private assessBacklog(remaining: number): BacklogPressure {
    if (remaining === 0) {
      return 'cleared';
    }

    if (remaining <= 3) {
      return 'low';
    }

    if (remaining <= 10) {
      return 'moderate';
    }

    return 'high';
  }

  private assessAccuracy(accuracyRate: number): AccuracyRisk {
    if (accuracyRate >= 0.9) {
      return 'stable';
    }

    if (accuracyRate >= 0.8) {
      return 'watch';
    }

    return 'critical';
  }

  private deriveRecommendation(input: RecommendationContext): Recommendation {
    const matchingRule = RECOMMENDATION_RULES.find((rule) => {
      if (rule.backlog && rule.backlog !== input.backlog) {
        return false;
      }

      if (rule.accuracy && rule.accuracy !== input.accuracy) {
        return false;
      }

      if (rule.predicate && !rule.predicate(input)) {
        return false;
      }

      return true;
    });

    return matchingRule?.recommendation ?? RECOMMENDATION_FALLBACK;
  }
}
