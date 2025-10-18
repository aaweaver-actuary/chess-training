use serde::{Deserialize, Serialize};

use crate::errors::AdapterResult;
use crate::state::{AttemptResult, QuizSummary, StepMetadata};

/// Trait describing how adapters interact with the quiz engine.
pub trait QuizPort {
    /// Presents a quiz prompt and collects the learner's SAN response.
    ///
    /// # Errors
    ///
    /// Implementations should return [`crate::errors::QuizError::Io`] when underlying I/O
    /// operations fail.
    fn present_prompt(&mut self, context: PromptContext) -> AdapterResult<String>;

    /// Emits feedback reflecting the outcome of the most recent attempt.
    ///
    /// # Errors
    ///
    /// Implementations should return [`crate::errors::QuizError::Io`] when emitting feedback
    /// encounters I/O failures.
    fn publish_feedback(&mut self, feedback: FeedbackMessage) -> AdapterResult<()>;

    /// Shares the final summary once the quiz completes.
    ///
    /// # Errors
    ///
    /// Implementations should return [`crate::errors::QuizError::Io`] when summary delivery
    /// fails due to adapter I/O.
    fn present_summary(&mut self, summary: &QuizSummary) -> AdapterResult<()>;
}

/// Context supplied to adapters when prompting for the next SAN move.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptContext {
    /// Zero-based index for the step currently being presented.
    pub step_index: usize,
    /// Total number of steps within the active session.
    pub total_steps: usize,
    /// FEN board snapshot that adapters can render before requesting input.
    pub board_fen: String,
    /// SAN prompt communicated to the learner.
    pub prompt_san: String,
    /// Optional SAN of the immediately prior move.
    pub previous_move_san: Option<String>,
    /// Number of retries remaining for the current step.
    pub remaining_retries: u8,
    /// Metadata describing the repertoire linkage and theme for the step.
    pub metadata: StepMetadata,
}

impl PromptContext {
    /// Returns the human-friendly (1-indexed) move number.
    #[must_use]
    pub fn display_index(&self) -> usize {
        self.step_index + 1
    }
}

/// Feedback delivered to adapters after an attempt is graded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedbackMessage {
    /// Zero-based index of the step being reported.
    pub step_index: usize,
    /// Outcome of the learner's attempt.
    pub result: AttemptResult,
    /// Learner response captured for this attempt, if any.
    pub learner_response: Option<String>,
    /// Canonical SAN solution for the step.
    pub solution_san: String,
    /// Annotations that may accompany the final reveal.
    pub annotations: Vec<String>,
    /// Remaining retries after processing the attempt.
    pub remaining_retries: u8,
    /// Metadata describing the repertoire linkage and theme for the step.
    pub metadata: StepMetadata,
}

impl FeedbackMessage {
    /// Convenience constructor for successful attempts.
    #[must_use]
    pub fn success(
        step_index: usize,
        learner_response: impl Into<String>,
        annotations: Vec<String>,
        metadata: StepMetadata,
    ) -> Self {
        Self {
            step_index,
            result: AttemptResult::Correct,
            learner_response: Some(learner_response.into()),
            solution_san: String::new(),
            annotations,
            remaining_retries: 0,
            metadata,
        }
    }

    /// Convenience constructor for incorrect attempts with remaining retries.
    #[must_use]
    pub fn retry(
        step_index: usize,
        learner_response: impl Into<String>,
        remaining_retries: u8,
        metadata: StepMetadata,
    ) -> Self {
        Self {
            step_index,
            result: AttemptResult::Pending,
            learner_response: Some(learner_response.into()),
            solution_san: String::new(),
            annotations: Vec::new(),
            remaining_retries,
            metadata,
        }
    }

    /// Convenience constructor for final incorrect attempts.
    #[must_use]
    pub fn failure(
        step_index: usize,
        learner_response: Option<String>,
        solution_san: impl Into<String>,
        annotations: Vec<String>,
        metadata: StepMetadata,
    ) -> Self {
        Self {
            step_index,
            result: AttemptResult::Incorrect,
            learner_response,
            solution_san: solution_san.into(),
            annotations,
            remaining_retries: 0,
            metadata,
        }
    }
}

#[cfg(all(test, feature = "cli"))]
mod tests {
    use super::*;
    use crate::state::QuizSummary;
    use std::io::Cursor;
    use std::io::{self, Write};

    use crate::cli::TerminalPort;
    use crate::errors::QuizError;

    fn context() -> PromptContext {
        PromptContext {
            step_index: 0,
            total_steps: 2,
            board_fen: "8/8/8/8/8/8/8/8 w - - 0 1".into(),
            prompt_san: "Qh5+".into(),
            previous_move_san: Some("Nc6".into()),
            remaining_retries: 1,
            metadata: StepMetadata {
                step_id: Some("quiz-step-1".into()),
                card_ref: Some("card-123".into()),
                themes: vec!["attack".into(), "mate".into()],
            },
        }
    }

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("writer failed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("flush failed"))
        }
    }

    #[test]
    fn terminal_port_surfaces_io_errors_via_quiz_error() {
        let input = Cursor::new("e4\n");
        let writer = FailingWriter;
        let mut port = TerminalPort::with_io(input, writer);

        let error = port
            .present_prompt(context())
            .expect_err("writer failure should surface as QuizError");

        assert_eq!(error, QuizError::Io);
    }

    #[test]
    fn prompt_context_display_index_is_1_indexed() {
        let mut ctx = context();
        ctx.step_index = 4;
        assert_eq!(ctx.display_index(), 5);
    }

    #[test]
    fn feedback_message_constructors_cover_all_variants() {
        let metadata = StepMetadata::canonical_for_index(0);
        let success = FeedbackMessage::success(0, "Qh5+", vec!["mate".into()], metadata.clone());
        assert_eq!(success.result, AttemptResult::Correct);
        assert_eq!(success.learner_response.as_deref(), Some("Qh5+"));
        assert_eq!(success.annotations, vec!["mate".to_string()]);
        assert_eq!(success.remaining_retries, 0);
        assert_eq!(success.metadata.step_id.as_deref(), Some("quiz-step-1"));

        let retry = FeedbackMessage::retry(1, "Qh4", 2, metadata.clone());
        assert_eq!(retry.result, AttemptResult::Pending);
        assert_eq!(retry.learner_response.as_deref(), Some("Qh4"));
        assert_eq!(retry.remaining_retries, 2);
        assert!(retry.annotations.is_empty());
        assert_eq!(retry.metadata.step_id.as_deref(), Some("quiz-step-1"));

        let failure = FeedbackMessage::failure(2, None, "Qh5+", vec!["skewer".into()], metadata);
        assert_eq!(failure.result, AttemptResult::Incorrect);
        assert_eq!(failure.learner_response, None);
        assert_eq!(failure.solution_san, "Qh5+");
        assert_eq!(failure.annotations, vec!["skewer".to_string()]);
        assert_eq!(failure.remaining_retries, 0);
        assert_eq!(failure.metadata.step_id.as_deref(), Some("quiz-step-1"));
    }

    #[test]
    fn terminal_port_prompts_and_reads_trimmed_response() {
        let input = Cursor::new("Nf3 \n");
        let writer = Vec::new();
        let mut port = TerminalPort::with_io(input, writer);

        let response = port
            .present_prompt(context())
            .expect("terminal prompt should succeed");
        assert_eq!(response, "Nf3");

        let (_, writer) = port.into_inner();
        let output = String::from_utf8(writer).expect("utf8");
        assert!(output.contains("Move 1/2"));
        assert!(output.contains("Qh5+"));
        assert!(output.contains("Previous move: Nc6"));
        assert!(output.contains("Step ID: quiz-step-1"));
        assert!(output.contains("Card ref: card-123"));
        assert!(output.contains("Themes: attack, mate"));
    }

    #[test]
    fn terminal_port_prompts_without_previous_move_or_retries() {
        let input = Cursor::new("Nf3\n");
        let writer = Vec::new();
        let mut port = TerminalPort::with_io(input, writer);

        let mut ctx = context();
        ctx.previous_move_san = None;
        ctx.remaining_retries = 0;

        let response = port
            .present_prompt(ctx)
            .expect("terminal prompt should succeed");
        assert_eq!(response, "Nf3");

        let (_, writer) = port.into_inner();
        let output = String::from_utf8(writer).expect("utf8");
        assert!(output.contains("Board FEN"));
        assert!(!output.contains("Previous move"));
        assert!(!output.contains("Retries remaining"));
    }

    #[test]
    fn terminal_port_renders_success_feedback() {
        let input = Cursor::new(String::new());
        let writer = Vec::new();
        let mut port = TerminalPort::with_io(input, writer);

        let message = FeedbackMessage {
            step_index: 0,
            result: AttemptResult::Correct,
            learner_response: Some("Qh5+".into()),
            solution_san: "Qh5+".into(),
            annotations: vec!["Classic Scholar's Mate pattern".into()],
            remaining_retries: 1,
        };

        port.publish_feedback(message)
            .expect("feedback output should succeed");

        let (_, writer) = port.into_inner();
        let output = String::from_utf8(writer).expect("utf8");
        assert!(output.contains("Correct!"));
        assert!(output.contains("Classic Scholar's Mate pattern"));
    }

    #[test]
    fn terminal_port_renders_retry_feedback_with_response() {
        let input = Cursor::new(String::new());
        let writer = Vec::new();
        let mut port = TerminalPort::with_io(input, writer);

        let message = FeedbackMessage::retry(0, "Qh5", 0);

        port.publish_feedback(message)
            .expect("feedback output should succeed");

        let (_, writer) = port.into_inner();
        let output = String::from_utf8(writer).expect("utf8");
        assert!(output.contains("Incorrect, try again."));
        assert!(output.contains("Retries remaining: 0"));
        assert!(output.contains("Your answer: Qh5"));
    }

    #[test]
    fn terminal_port_renders_failure_feedback_with_solution() {
        let input = Cursor::new(String::new());
        let writer = Vec::new();
        let mut port = TerminalPort::with_io(input, writer);

        let message = FeedbackMessage {
            step_index: 1,
            result: AttemptResult::Incorrect,
            learner_response: Some("Qh4".into()),
            solution_san: "Qh5+".into(),
            annotations: vec![],
            remaining_retries: 0,
        };

        port.publish_feedback(message)
            .expect("feedback output should succeed");

        let (_, writer) = port.into_inner();
        let output = String::from_utf8(writer).expect("utf8");
        assert!(output.contains("Incorrect."));
        assert!(output.contains("Solution: Qh5+"));
    }

    #[test]
    fn terminal_port_renders_failure_annotations() {
        let input = Cursor::new(String::new());
        let writer = Vec::new();
        let mut port = TerminalPort::with_io(input, writer);

        let message =
            FeedbackMessage::failure(0, Some("Qh4".into()), "Qh5+", vec!["Fork the king".into()]);

        port.publish_feedback(message)
            .expect("feedback output should succeed");

        let (_, writer) = port.into_inner();
        let output = String::from_utf8(writer).expect("utf8");
        assert!(output.contains("Annotations:"));
        assert!(output.contains("- Fork the king"));
    }

    #[test]
    fn terminal_port_prints_summary() {
        let input = Cursor::new(String::new());
        let writer = Vec::new();
        let mut port = TerminalPort::with_io(input, writer);

        let summary = QuizSummary {
            total_steps: 2,
            completed_steps: 2,
            correct_answers: 1,
            incorrect_answers: 1,
            retries_consumed: 1,
        };

        port.present_summary(&summary)
            .expect("summary output should succeed");

        let (_, writer) = port.into_inner();
        let output = String::from_utf8(writer).expect("utf8");
        assert!(output.contains("Quiz complete"));
        assert!(output.contains("Correct: 1"));
        assert!(output.contains("Incorrect: 1"));
        assert!(output.contains("Retries used: 1"));
    }
}
