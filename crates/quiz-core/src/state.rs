#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Immutable snapshot of a learner's progress through a chess quiz.
///
/// The session keeps track of each `QuizStep`, the active index the engine is
/// presenting, and the running [`QuizSummary`] totals that adapters can render
/// after completion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuizSession {
    /// Ordered collection of prompts and attempts that make up the quiz.
    pub steps: Vec<QuizStep>,
    /// Index of the step currently presented to the learner.
    pub current_index: usize,
    /// Aggregate scoring and retry information captured as the quiz advances.
    pub summary: QuizSummary,
}

impl QuizSession {
    /// Constructs a session from prepared steps, initialising summary totals to
    /// align with the number of steps and resetting the active index to zero.
    pub fn new(steps: Vec<QuizStep>) -> Self {
        let summary = QuizSummary::new(steps.len());

        Self {
            steps,
            current_index: 0,
            summary,
        }
    }

    /// Returns `true` when all steps have been attempted.
    pub fn is_complete(&self) -> bool {
        self.current_index >= self.steps.len()
    }

    /// Provides a reference to the currently active step, if any remain.
    pub fn current_step(&self) -> Option<&QuizStep> {
        self.steps.get(self.current_index)
    }
}

/// Encapsulates the context required to prompt the learner for a move.
///
/// Each step stores the board position in Forsyth–Edwards Notation (FEN), the
/// SAN move the learner is expected to supply, and the [`AttemptState`] tracking
/// retries and responses. Optional annotations may be surfaced after the step
/// completes so adapters can display coaching notes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuizStep {
    /// Board snapshot before the learner's move, serialised as a FEN string.
    pub board_fen: String,
    /// Algebraic (SAN) prompt presented to the learner.
    pub prompt_san: String,
    /// The canonical SAN solution revealed after a final attempt.
    pub solution_san: String,
    /// Tracking state for learner attempts, retries, and captured responses.
    pub attempt: AttemptState,
    /// Optional annotations that accompany the step once graded.
    pub annotations: Vec<String>,
}

impl QuizStep {
    /// Creates a new step with the provided board snapshot and SAN prompts.
    ///
    /// The `max_retries` parameter configures how many retries the learner is
    /// allowed before the step is marked incorrect.
    pub fn new(
        board_fen: impl Into<String>,
        prompt_san: impl Into<String>,
        solution_san: impl Into<String>,
        max_retries: u8,
    ) -> Self {
        Self {
            board_fen: board_fen.into(),
            prompt_san: prompt_san.into(),
            solution_san: solution_san.into(),
            attempt: AttemptState::new(max_retries),
            annotations: Vec::new(),
        }
    }
}

/// Represents the current attempt status for a single quiz step.
///
/// Tracks retries and learner responses so the engine can enforce retry
/// budgets and surfaces the final outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttemptState {
    /// High-level outcome recorded once the learner either succeeds or fails.
    pub result: AttemptResult,
    /// Number of retries allowed beyond the initial attempt.
    pub retries_allowed: u8,
    /// Number of retries already consumed.
    pub retries_used: u8,
    /// History of SAN responses submitted by the learner.
    pub responses: Vec<String>,
}

impl AttemptState {
    /// Creates a pending attempt state configured with a retry allowance.
    pub fn new(max_retries: u8) -> Self {
        Self {
            result: AttemptResult::Pending,
            retries_allowed: max_retries,
            retries_used: 0,
            responses: Vec::new(),
        }
    }

    /// Calculates how many retries remain available to the learner.
    pub fn remaining_retries(&self) -> u8 {
        self.retries_allowed.saturating_sub(self.retries_used)
    }
}

/// Final scoring summary produced once the session concludes.
///
/// Stores totals for correct/incorrect answers and the number of retries
/// consumed so analytics and adapters can present aggregate outcomes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuizSummary {
    /// Total number of steps included in the quiz session.
    pub total_steps: usize,
    /// Number of steps the learner has completed (correct or incorrect).
    pub completed_steps: usize,
    /// Count of steps answered correctly.
    pub correct_answers: usize,
    /// Count of steps answered incorrectly after exhausting retries.
    pub incorrect_answers: usize,
    /// Total number of retries consumed across all steps.
    pub retries_consumed: usize,
}

impl QuizSummary {
    /// Prepares a summary for a quiz with the specified number of steps.
    pub fn new(total_steps: usize) -> Self {
        Self {
            total_steps,
            ..Self::default()
        }
    }
}

impl Default for QuizSummary {
    fn default() -> Self {
        Self {
            total_steps: 0,
            completed_steps: 0,
            correct_answers: 0,
            incorrect_answers: 0,
            retries_consumed: 0,
        }
    }
}

/// Outcome state for a learner's attempt at a given quiz step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttemptResult {
    /// The learner has not yet provided a conclusive answer.
    Pending,
    /// The learner supplied the correct SAN and the step is complete.
    Correct,
    /// The learner exhausted retries or revealed the move incorrectly.
    Incorrect,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_step(max_retries: u8) -> QuizStep {
        QuizStep::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "?",
            "e4",
            max_retries,
        )
    }

    #[test]
    fn session_initialises_summary_totals() {
        let steps = vec![sample_step(1), sample_step(2)];
        let session = QuizSession::new(steps.clone());

        assert_eq!(session.steps, steps);
        assert_eq!(session.current_index, 0);
        assert_eq!(session.summary.total_steps, steps.len());
        assert_eq!(session.summary.completed_steps, 0);
    }

    #[test]
    fn attempt_state_tracks_remaining_retries() {
        let mut attempt = AttemptState::new(1);
        assert_eq!(attempt.remaining_retries(), 1);

        attempt.retries_used = 1;
        assert_eq!(attempt.remaining_retries(), 0);
    }

    #[test]
    fn quiz_step_initialises_attempt_state() {
        let step = sample_step(2);

        assert_eq!(step.attempt.retries_allowed, 2);
        assert_eq!(step.attempt.retries_used, 0);
        assert_eq!(step.attempt.result, AttemptResult::Pending);
    }

    #[test]
    fn summary_constructor_sets_totals() {
        let summary = QuizSummary::new(5);

        assert_eq!(summary.total_steps, 5);
        assert_eq!(summary.completed_steps, 0);
        assert_eq!(summary.correct_answers, 0);
        assert_eq!(summary.incorrect_answers, 0);
        assert_eq!(summary.retries_consumed, 0);
    }
}
