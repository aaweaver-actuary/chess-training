use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpcomingUnlock {
    pub id: String,
    #[serde(rename = "move")]
    pub move_text: String,
    pub idea: String,
    pub scheduled_for: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSnapshot {
    pub due_cards: i32,
    pub completed_cards: i32,
    pub accuracy_rate: f64,
    pub streak_length: u32,
    pub upcoming_unlocks: Vec<UpcomingUnlock>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BacklogPressure {
    Cleared,
    Low,
    Moderate,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccuracyRisk {
    Stable,
    Watch,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressOverview {
    pub total_due: u32,
    pub completed_today: u32,
    pub remaining: u32,
    pub completion_rate: f64,
    pub accuracy_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TensionOverview {
    pub backlog_pressure: BacklogPressure,
    pub accuracy_risk: AccuracyRisk,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recommendation {
    pub primary_action: String,
    pub secondary_action: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewOverview {
    pub progress: ProgressOverview,
    pub tension: TensionOverview,
    pub recommendation: Recommendation,
    pub upcoming_unlocks: Vec<UpcomingUnlock>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ReviewPlannerError {
    #[error("Review counts cannot be negative")]
    NegativeCounts,
    #[error("Accuracy must be between 0 and 1")]
    InvalidAccuracy,
}

#[derive(Debug, Default)]
pub struct ReviewPlanner;

impl ReviewPlanner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Builds an overview from the provided snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`ReviewPlannerError::NegativeCounts`] when the snapshot contains negative review
    /// counts or [`ReviewPlannerError::InvalidAccuracy`] when the accuracy rate falls outside the
    /// inclusive `[0, 1]` range.
    pub fn build_overview(
        &self,
        snapshot: &ReviewSnapshot,
    ) -> Result<ReviewOverview, ReviewPlannerError> {
        if !(0.0..=1.0).contains(&snapshot.accuracy_rate) {
            return Err(ReviewPlannerError::InvalidAccuracy);
        }

        let due_cards =
            u32::try_from(snapshot.due_cards).map_err(|_| ReviewPlannerError::NegativeCounts)?;
        let completed_cards = u32::try_from(snapshot.completed_cards)
            .map_err(|_| ReviewPlannerError::NegativeCounts)?;
        let completed_today = completed_cards.min(due_cards);
        let remaining = due_cards.saturating_sub(completed_cards);
        let completion_rate = if due_cards == 0 {
            1.0
        } else {
            f64::from(completed_cards) / f64::from(due_cards)
        };

        let backlog_pressure = Self::assess_backlog(remaining);
        let accuracy_risk = Self::assess_accuracy(snapshot.accuracy_rate);
        let recommendation = Self::derive_recommendation(RecommendationContext {
            backlog: backlog_pressure,
            accuracy: accuracy_risk,
            streak_length: snapshot.streak_length,
            remaining,
        });

        Ok(ReviewOverview {
            progress: ProgressOverview {
                total_due: due_cards,
                completed_today,
                remaining,
                completion_rate,
                accuracy_rate: snapshot.accuracy_rate,
            },
            tension: TensionOverview {
                backlog_pressure,
                accuracy_risk,
            },
            recommendation,
            upcoming_unlocks: snapshot.upcoming_unlocks.clone(),
        })
    }

    fn assess_backlog(remaining: u32) -> BacklogPressure {
        match remaining {
            0 => BacklogPressure::Cleared,
            1..=3 => BacklogPressure::Low,
            4..=10 => BacklogPressure::Moderate,
            _ => BacklogPressure::High,
        }
    }

    fn assess_accuracy(accuracy_rate: f64) -> AccuracyRisk {
        if accuracy_rate >= 0.9 {
            AccuracyRisk::Stable
        } else if accuracy_rate >= 0.8 {
            AccuracyRisk::Watch
        } else {
            AccuracyRisk::Critical
        }
    }

    fn derive_recommendation(context: RecommendationContext) -> Recommendation {
        RECOMMENDATION_RULES
            .iter()
            .find(|rule| rule.matches(&context))
            .map_or_else(
                || RECOMMENDATION_FALLBACK.to_owned(),
                |rule| rule.template.to_owned(),
            )
    }
}

#[derive(Debug, Clone, Copy)]
struct RecommendationContext {
    backlog: BacklogPressure,
    accuracy: AccuracyRisk,
    streak_length: u32,
    remaining: u32,
}

#[derive(Debug, Clone, Copy)]
struct RecommendationRule {
    backlog: Option<BacklogPressure>,
    accuracy: Option<AccuracyRisk>,
    predicate: Option<fn(&RecommendationContext) -> bool>,
    template: RecommendationTemplate,
}

impl RecommendationRule {
    fn matches(&self, context: &RecommendationContext) -> bool {
        if self
            .backlog
            .is_some_and(|expected| expected != context.backlog)
        {
            return false;
        }

        if self
            .accuracy
            .is_some_and(|expected| expected != context.accuracy)
        {
            return false;
        }

        if self.predicate.is_some_and(|predicate| !predicate(context)) {
            return false;
        }

        true
    }
}

#[derive(Debug, Clone, Copy)]
struct RecommendationTemplate {
    primary_action: &'static str,
    secondary_action: &'static str,
}

impl RecommendationTemplate {
    fn to_owned(self) -> Recommendation {
        Recommendation {
            primary_action: self.primary_action.to_string(),
            secondary_action: self.secondary_action.to_string(),
        }
    }
}

const RECOMMENDATION_RULES: &[RecommendationRule] = &[
    RecommendationRule {
        backlog: Some(BacklogPressure::High),
        accuracy: None,
        predicate: None,
        template: RecommendationTemplate {
            primary_action: "Catch up on overdue reviews",
            secondary_action: "Reinforce accuracy with short tactics drills",
        },
    },
    RecommendationRule {
        backlog: Some(BacklogPressure::Moderate),
        accuracy: None,
        predicate: None,
        template: RecommendationTemplate {
            primary_action: "Work through today's reviews in two focused blocks",
            secondary_action: "Log any mistakes immediately to revisit tomorrow",
        },
    },
    RecommendationRule {
        backlog: Some(BacklogPressure::Low),
        accuracy: Some(AccuracyRisk::Critical),
        predicate: None,
        template: RecommendationTemplate {
            primary_action: "Stabilize accuracy with quick refresh drills",
            secondary_action: "Tag the weakest lines for focused review",
        },
    },
    RecommendationRule {
        backlog: Some(BacklogPressure::Low),
        accuracy: None,
        predicate: None,
        template: RecommendationTemplate {
            primary_action: "Complete the remaining reviews in a single sprint",
            secondary_action: "Do a light skim of yesterday's problem areas",
        },
    },
    RecommendationRule {
        backlog: None,
        accuracy: Some(AccuracyRisk::Critical),
        predicate: None,
        template: RecommendationTemplate {
            primary_action: "Rebuild confidence on the weakest variations",
            secondary_action: "Schedule a tactics-only session for reinforcement",
        },
    },
    RecommendationRule {
        backlog: None,
        accuracy: Some(AccuracyRisk::Watch),
        predicate: None,
        template: RecommendationTemplate {
            primary_action: "Finish the day with one more focused review block",
            secondary_action: "Revisit the last set of inaccuracies to lock them in",
        },
    },
    RecommendationRule {
        backlog: None,
        accuracy: None,
        predicate: Some(|context: &RecommendationContext| {
            context.streak_length >= 10 && context.remaining == 0
        }),
        template: RecommendationTemplate {
            primary_action: "Add one new line to your repertoire",
            secondary_action: "Review high-value mistakes from the past week",
        },
    },
];

const RECOMMENDATION_FALLBACK: RecommendationTemplate = RecommendationTemplate {
    primary_action: "Plan tomorrow's unlock and keep the momentum",
    secondary_action: "Share today's success in your training journal",
};

#[cfg(test)]
mod tests {
    use super::*;

    fn base_snapshot() -> ReviewSnapshot {
        ReviewSnapshot {
            due_cards: 20,
            completed_cards: 5,
            accuracy_rate: 0.6,
            streak_length: 3,
            upcoming_unlocks: vec![
                UpcomingUnlock {
                    id: "unlock-1".to_string(),
                    move_text: "e4".to_string(),
                    idea: "King pawn opening control".to_string(),
                    scheduled_for: "2024-01-10".to_string(),
                },
                UpcomingUnlock {
                    id: "unlock-2".to_string(),
                    move_text: "Nf3".to_string(),
                    idea: "Attack e5 pawn".to_string(),
                    scheduled_for: "2024-01-12".to_string(),
                },
            ],
        }
    }

    #[test]
    fn summarizes_progress_and_backlog_pressure() {
        let planner = ReviewPlanner::new();
        let snapshot = base_snapshot();
        let overview = planner.build_overview(&snapshot).expect("valid overview");

        assert_eq!(overview.progress.total_due, 20);
        assert_eq!(overview.progress.completed_today, 5);
        assert_eq!(overview.progress.remaining, 15);
        assert!((overview.progress.completion_rate - 0.25).abs() < f64::EPSILON);
        assert_eq!(overview.tension.backlog_pressure, BacklogPressure::High);
        assert_eq!(
            overview.recommendation.primary_action,
            "Catch up on overdue reviews"
        );
        assert!(
            overview
                .recommendation
                .secondary_action
                .contains("Reinforce accuracy with short tactics drills")
        );
    }

    #[test]
    fn detects_strong_day_encourages_expansion() {
        let planner = ReviewPlanner::new();
        let snapshot = ReviewSnapshot {
            due_cards: 6,
            completed_cards: 6,
            accuracy_rate: 0.92,
            streak_length: 12,
            upcoming_unlocks: base_snapshot().upcoming_unlocks,
        };

        let overview = planner.build_overview(&snapshot).expect("valid overview");

        assert_eq!(overview.progress.remaining, 0);
        assert_eq!(overview.tension.backlog_pressure, BacklogPressure::Cleared);
        assert_eq!(
            overview.recommendation.primary_action,
            "Add one new line to your repertoire"
        );
    }

    #[test]
    fn low_backlog_with_critical_accuracy_prioritizes_stabilizing() {
        let planner = ReviewPlanner::new();
        let snapshot = ReviewSnapshot {
            due_cards: 4,
            completed_cards: 3,
            accuracy_rate: 0.68,
            streak_length: 2,
            upcoming_unlocks: vec![],
        };

        let overview = planner.build_overview(&snapshot).expect("valid overview");

        assert_eq!(overview.tension.backlog_pressure, BacklogPressure::Low);
        assert_eq!(overview.tension.accuracy_risk, AccuracyRisk::Critical);
        assert_eq!(
            overview.recommendation.primary_action,
            "Stabilize accuracy with quick refresh drills"
        );
    }

    #[test]
    fn fallback_is_used_when_no_rule_matches() {
        let planner = ReviewPlanner::new();
        let snapshot = ReviewSnapshot {
            due_cards: 5,
            completed_cards: 5,
            accuracy_rate: 0.95,
            streak_length: 1,
            upcoming_unlocks: vec![],
        };

        let overview = planner.build_overview(&snapshot).expect("valid overview");

        assert_eq!(overview.tension.backlog_pressure, BacklogPressure::Cleared);
        assert_eq!(overview.tension.accuracy_risk, AccuracyRisk::Stable);
        assert_eq!(
            overview.recommendation.primary_action,
            "Plan tomorrow's unlock and keep the momentum"
        );
    }

    #[test]
    fn raises_on_invalid_snapshot() {
        let planner = ReviewPlanner::new();
        let mut snapshot = base_snapshot();
        snapshot.due_cards = -1;
        let err = planner.build_overview(&snapshot).unwrap_err();
        assert_eq!(err, ReviewPlannerError::NegativeCounts);

        snapshot.due_cards = 10;
        snapshot.accuracy_rate = 1.2;
        let err = planner.build_overview(&snapshot).unwrap_err();
        assert_eq!(err, ReviewPlannerError::InvalidAccuracy);
    }
}
