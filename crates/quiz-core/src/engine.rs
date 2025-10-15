use crate::errors::QuizResult;
use crate::ports::{FeedbackMessage, PromptContext, QuizPort};
use crate::source::QuizSource;
use crate::state::{AttemptResult, QuizSession, QuizStep, QuizSummary};

/// Orchestrates quiz sessions by coordinating prompts, retries, and summaries.
pub struct QuizEngine {
    session: QuizSession,
}

impl QuizEngine {
    /// Creates a new engine from an existing [`QuizSession`].
    #[must_use]
    pub fn new(session: QuizSession) -> Self {
        Self { session }
    }

    /// Builds an engine from a pre-parsed [`QuizSource`].
    #[must_use]
    pub fn from_source(source: &QuizSource, max_retries: u8) -> Self {
        Self::new(QuizSession::from_source(source, max_retries))
    }

    /// Parses PGN text into a quiz engine ready to run.
    ///
    /// # Errors
    /// Returns an error when the PGN text cannot be parsed into a valid quiz.
    pub fn from_pgn(pgn: &str, max_retries: u8) -> QuizResult<Self> {
        Ok(Self::new(QuizSession::from_pgn(pgn, max_retries)?))
    }

    /// Runs the quiz using the supplied adapter port.
    ///
    /// # Errors
    /// Propagates any adapter or grading errors encountered while running the quiz.
    pub fn run<P: QuizPort>(&mut self, port: &mut P) -> QuizResult<&QuizSummary> {
        while !self.session.is_complete() {
            self.process_current_step(port)?;
        }

        port.present_summary(&self.session.summary)?;
        Ok(&self.session.summary)
    }

    fn process_current_step<P: QuizPort>(&mut self, port: &mut P) -> QuizResult<()> {
        loop {
            let step_index = self.session.current_index;
            let total_steps = self.session.steps.len();
            let previous_move = if step_index == 0 {
                None
            } else {
                Some(self.session.steps[step_index - 1].solution_san.clone())
            };

            let (board_fen, prompt_san, remaining_retries) = {
                let step = &self.session.steps[step_index];
                (
                    step.board_fen.clone(),
                    step.prompt_san.clone(),
                    step.attempt.remaining_retries(),
                )
            };

            let context = PromptContext {
                step_index,
                total_steps,
                board_fen,
                prompt_san,
                previous_move_san: previous_move,
                remaining_retries,
            };

            let response = port.present_prompt(context)?;

            let GradeOutcome {
                feedback,
                final_result,
            } = {
                let step = &mut self.session.steps[step_index];
                Self::grade_attempt(step_index, step, &response)
            };

            port.publish_feedback(feedback)?;

            if let Some(result) = final_result {
                let retries_used = self.session.steps[step_index].attempt.retries_used as usize;
                self.session.summary.completed_steps += 1;
                self.session.summary.retries_consumed += retries_used;

                match result {
                    AttemptResult::Correct => self.session.summary.correct_answers += 1,
                    AttemptResult::Incorrect => self.session.summary.incorrect_answers += 1,
                    AttemptResult::Pending => {}
                }

                self.advance();
                break;
            }
        }

        Ok(())
    }

    /// Advances to the next step once the current step completes.
    fn advance(&mut self) {
        self.session.current_index += 1;
    }

    /// Grades an attempt and returns the corresponding feedback message.
    fn grade_attempt(step_index: usize, step: &mut QuizStep, response: &str) -> GradeOutcome {
        let trimmed = response.trim().to_string();
        step.attempt.responses.push(trimmed.clone());

        if san_matches(&trimmed, &step.solution_san) {
            step.attempt.result = AttemptResult::Correct;
            return GradeOutcome {
                feedback: FeedbackMessage::success(step_index, trimmed, step.annotations.clone()),
                final_result: Some(AttemptResult::Correct),
            };
        }

        let remaining = step.attempt.remaining_retries();
        if remaining > 0 {
            step.attempt.retries_used += 1;
            return GradeOutcome {
                feedback: FeedbackMessage::retry(step_index, trimmed, remaining),
                final_result: None,
            };
        }

        step.attempt.result = AttemptResult::Incorrect;
        GradeOutcome {
            feedback: FeedbackMessage::failure(
                step_index,
                (!trimmed.is_empty()).then_some(trimmed),
                step.solution_san.clone(),
                step.annotations.clone(),
            ),
            final_result: Some(AttemptResult::Incorrect),
        }
    }

    /// Provides read-only access to the underlying session for inspection.
    #[must_use]
    pub fn session(&self) -> &QuizSession {
        &self.session
    }
}

struct GradeOutcome {
    feedback: FeedbackMessage,
    final_result: Option<AttemptResult>,
}

fn san_matches(input: &str, solution: &str) -> bool {
    let normalised_input = input.trim();
    if normalised_input.is_empty() {
        return false;
    }

    normalised_input.eq_ignore_ascii_case(solution.trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::QuizError;
    use crate::ports::QuizPort;
    use std::collections::VecDeque;

    struct FakePort {
        responses: VecDeque<String>,
        prompts: Vec<PromptContext>,
        feedback: Vec<FeedbackMessage>,
        summary: Option<QuizSummary>,
        feedback_calls: usize,
        fail_feedback_after: Option<usize>,
        fail_summary: bool,
    }

    impl FakePort {
        fn with_responses(responses: Vec<&str>) -> Self {
            Self {
                responses: responses.into_iter().map(String::from).collect(),
                prompts: Vec::new(),
                feedback: Vec::new(),
                summary: None,
                feedback_calls: 0,
                fail_feedback_after: None,
                fail_summary: false,
            }
        }

        fn failing_feedback(responses: Vec<&str>) -> Self {
            let mut port = Self::with_responses(responses);
            port.fail_feedback_after = Some(1);
            port
        }

        fn failing_summary(responses: Vec<&str>) -> Self {
            let mut port = Self::with_responses(responses);
            port.fail_summary = true;
            port
        }
    }

    impl QuizPort for FakePort {
        fn present_prompt(&mut self, context: PromptContext) -> Result<String, QuizError> {
            self.prompts.push(context);
            self.responses.pop_front().ok_or(QuizError::Io)
        }

        fn publish_feedback(&mut self, feedback: FeedbackMessage) -> Result<(), QuizError> {
            self.feedback_calls += 1;

            if let Some(threshold) = self.fail_feedback_after
                && self.feedback_calls >= threshold
            {
                return Err(QuizError::Io);
            }

            self.feedback.push(feedback);
            Ok(())
        }

        fn present_summary(&mut self, summary: &QuizSummary) -> Result<(), QuizError> {
            if self.fail_summary {
                return Err(QuizError::Io);
            }

            self.summary = Some(summary.clone());
            Ok(())
        }
    }

    #[test]
    fn run_processes_correct_answers_and_publishes_summary() {
        let mut engine = QuizEngine::from_pgn("1. e4 e5 *", 1).expect("PGN should parse");
        let mut port = FakePort::with_responses(vec!["e4", "e5"]);

        let summary = engine.run(&mut port).expect("engine should complete");

        assert_eq!(summary.total_steps, 2);
        assert_eq!(summary.correct_answers, 2);
        assert_eq!(summary.incorrect_answers, 0);
        assert_eq!(port.feedback.len(), 2);
        assert!(
            port.feedback
                .iter()
                .all(|msg| msg.result == AttemptResult::Correct)
        );
        assert!(port.summary.is_some());
        assert_eq!(engine.session().current_index, 2);
        assert_eq!(port.prompts.len(), 2);
        assert!(port.prompts[0].previous_move_san.is_none());
        assert_eq!(port.prompts[1].previous_move_san.as_deref(), Some("e4"));
    }

    #[test]
    fn engine_allows_single_retry_and_tracks_consumed_retries() {
        let mut engine = QuizEngine::from_pgn("1. e4 *", 1).expect("PGN should parse");
        let mut port = FakePort::with_responses(vec!["d4", "e4"]);

        let summary = engine.run(&mut port).expect("engine should complete");

        assert_eq!(summary.correct_answers, 1);
        assert_eq!(summary.retries_consumed, 1);
        assert_eq!(port.feedback.len(), 2);
        assert_eq!(port.feedback[0].result, AttemptResult::Pending);
        assert_eq!(port.feedback[0].remaining_retries, 1);
        assert_eq!(port.feedback[1].result, AttemptResult::Correct);
        assert_eq!(port.prompts.len(), 2);
        assert_eq!(port.prompts[0].remaining_retries, 1);
        assert_eq!(port.prompts[1].remaining_retries, 0);
    }

    #[test]
    fn engine_marks_incorrect_after_retry_exhaustion() {
        let mut engine = QuizEngine::from_pgn("1. e4 *", 1).expect("PGN should parse");
        let mut port = FakePort::with_responses(vec!["d4", "c4"]);

        let summary = engine.run(&mut port).expect("engine should complete");

        assert_eq!(summary.correct_answers, 0);
        assert_eq!(summary.incorrect_answers, 1);
        assert_eq!(summary.retries_consumed, 1);
        assert_eq!(port.feedback.len(), 2);
        assert_eq!(port.feedback[1].result, AttemptResult::Incorrect);
        assert_eq!(port.feedback[1].solution_san, "e4");
        assert_eq!(port.prompts[1].remaining_retries, 0);
    }

    #[test]
    fn engine_bubbles_prompt_failures_without_advancing_state() {
        let mut engine = QuizEngine::from_pgn("1. e4 *", 1).expect("PGN should parse");
        let mut port = FakePort::with_responses(vec![]);

        let error = engine
            .run(&mut port)
            .expect_err("prompt failure should surface");

        assert_eq!(error, QuizError::Io);
        assert_eq!(engine.session().current_index, 0);
        assert_eq!(engine.session().summary.completed_steps, 0);
        assert!(engine.session().steps[0].attempt.responses.is_empty());
    }

    #[test]
    fn engine_stops_when_feedback_publication_errors() {
        let mut engine = QuizEngine::from_pgn("1. e4 *", 1).expect("PGN should parse");
        let mut port = FakePort::failing_feedback(vec!["e4"]);

        let error = engine
            .run(&mut port)
            .expect_err("feedback error should propagate");

        assert_eq!(error, QuizError::Io);
        assert_eq!(engine.session().summary.completed_steps, 0);
        assert_eq!(engine.session().current_index, 0);
        let attempt = &engine.session().steps[0].attempt;
        assert_eq!(attempt.responses, vec!["e4".to_string()]);
        assert_eq!(attempt.result, AttemptResult::Correct);
    }

    #[test]
    fn engine_preserves_summary_when_summary_delivery_fails() {
        let mut engine = QuizEngine::from_pgn("1. e4 e5 *", 1).expect("PGN should parse");
        let mut port = FakePort::failing_summary(vec!["e4", "e5"]);

        let error = engine
            .run(&mut port)
            .expect_err("summary delivery failure should propagate");

        assert_eq!(error, QuizError::Io);
        assert_eq!(engine.session().summary.correct_answers, 2);
        assert_eq!(engine.session().summary.completed_steps, 2);
        assert_eq!(engine.session().summary.retries_consumed, 0);
        assert!(port.summary.is_none());
    }

    #[test]
    fn engine_records_trimmed_responses_across_retries() {
        let mut engine = QuizEngine::from_pgn("1. e4 *", 1).expect("PGN should parse");
        let mut port = FakePort::with_responses(vec!["   d4  ", "  E4  "]);

        let summary = engine.run(&mut port).expect("engine should complete");

        assert_eq!(summary.retries_consumed, 1);
        assert_eq!(summary.correct_answers, 1);
        let attempt = &engine.session().steps[0].attempt;
        assert_eq!(attempt.responses, vec!["d4".to_string(), "E4".to_string()]);
    }
}
