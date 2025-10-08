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

  private deriveRecommendation(input: {
    backlog: BacklogPressure;
    accuracy: AccuracyRisk;
    streakLength: number;
    remaining: number;
  }): ReviewOverview['recommendation'] {
    if (input.backlog === 'high') {
      return {
        primaryAction: 'Catch up on overdue reviews',
        secondaryAction: 'Reinforce accuracy with short tactics drills',
      };
    }

    if (input.backlog === 'moderate') {
      return {
        primaryAction: "Work through today's reviews in two focused blocks",
        secondaryAction: 'Log any mistakes immediately to revisit tomorrow',
      };
    }

    if (input.backlog === 'low') {
      if (input.accuracy === 'critical') {
        return {
          primaryAction: 'Stabilize accuracy with quick refresh drills',
          secondaryAction: 'Tag the weakest lines for focused review',
        };
      }

      return {
        primaryAction: 'Complete the remaining reviews in a single sprint',
        secondaryAction: "Do a light skim of yesterday's problem areas",
      };
    }

    if (input.accuracy === 'critical') {
      return {
        primaryAction: 'Rebuild confidence on the weakest variations',
        secondaryAction: 'Schedule a tactics-only session for reinforcement',
      };
    }

    if (input.accuracy === 'watch') {
      return {
        primaryAction: 'Finish the day with one more focused review block',
        secondaryAction: 'Revisit the last set of inaccuracies to lock them in',
      };
    }

    if (input.streakLength >= 10 && input.remaining === 0) {
      return {
        primaryAction: 'Add one new line to your repertoire',
        secondaryAction: 'Review high-value mistakes from the past week',
      };
    }

    return {
      primaryAction: "Plan tomorrow's unlock and keep the momentum",
      secondaryAction: "Share today's success in your training journal",
    };
  }
}
