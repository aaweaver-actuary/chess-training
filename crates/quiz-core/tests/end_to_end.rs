use std::collections::VecDeque;

use quiz_core::{
    AttemptResult, FeedbackMessage, PromptContext, QuizEngine, QuizError, QuizPort, QuizSummary,
};

/// Test harness that simulates a [`QuizPort`] by replaying a fixed
/// sequence of learner responses.
///
/// The engine under test interacts with this port exactly as it would with a
/// real UI adapter: prompts are handed to [`QuizPort::present_prompt`],
/// feedback is delivered via [`QuizPort::publish_feedback`], and the run
/// concludes with [`QuizPort::present_summary`].
///
/// Each interaction is recorded so that the integration tests in this module
/// can assert on the engine's behaviour without relying on I/O. When the
/// predetermined responses are exhausted the port surfaces [`QuizError::Io`],
/// mimicking a disconnected or unresponsive client.
struct DeterministicPort {
    responses: VecDeque<String>,
    pub prompts: Vec<PromptContext>,
    pub feedback: Vec<FeedbackMessage>,
    pub summary: Option<QuizSummary>,
}

impl DeterministicPort {
    /// Constructs a [`DeterministicPort`] that will yield the provided
    /// `responses` in order before signalling [`QuizError::Io`].
    fn new<I, S>(responses: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let responses = responses
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();

        Self {
            responses: VecDeque::from(responses),
            prompts: Vec::new(),
            feedback: Vec::new(),
            summary: None,
        }
    }
}

impl QuizPort for DeterministicPort {
    fn present_prompt(&mut self, context: PromptContext) -> Result<String, QuizError> {
        self.prompts.push(context);
        self.responses
            .pop_front()
            .ok_or(QuizError::Io)
    }

    fn publish_feedback(&mut self, feedback: FeedbackMessage) -> Result<(), QuizError> {
        self.feedback.push(feedback);
        Ok(())
    }

    fn present_summary(&mut self, summary: &QuizSummary) -> Result<(), QuizError> {
        self.summary = Some(summary.clone());
        Ok(())
    }
}

#[test]
fn perfect_run_records_summary_and_feedback() {
    let mut engine = QuizEngine::from_pgn("1. e4 e5 2. Nf3 Nc6 *", 1).expect("PGN should parse");
    let mut port = DeterministicPort::new(["e4", "e5", "Nf3", "Nc6"]);

    let summary = engine.run(&mut port).expect("engine should complete");

    assert_eq!(summary.total_steps, 4);
    assert_eq!(summary.completed_steps, 4);
    assert_eq!(summary.correct_answers, 4);
    assert_eq!(summary.incorrect_answers, 0);
    assert_eq!(summary.retries_consumed, 0);
    assert_eq!(port.feedback.len(), 4);
    assert!(port
        .feedback
        .iter()
        .all(|message| message.result == AttemptResult::Correct));
    assert_eq!(port.summary.as_ref(), Some(summary));
    assert_eq!(port.prompts.len(), 4);
    assert!(port.prompts.iter().all(|prompt| prompt.remaining_retries == 1));
}

#[test]
fn retry_then_success_flow_consumes_single_retry() {
    let mut engine = QuizEngine::from_pgn("1. e4 e5 *", 1).expect("PGN should parse");
    let mut port = DeterministicPort::new(["d4", "e4", "e5"]);

    let summary = engine.run(&mut port).expect("engine should complete");

    assert_eq!(summary.correct_answers, 2);
    assert_eq!(summary.incorrect_answers, 0);
    assert_eq!(summary.retries_consumed, 1);
    assert_eq!(summary.completed_steps, 2);

    assert_eq!(port.prompts.len(), 3);
    assert_eq!(port.prompts[0].step_index, 0);
    assert_eq!(port.prompts[0].remaining_retries, 1);
    assert_eq!(port.prompts[1].step_index, 0);
    assert_eq!(port.prompts[1].remaining_retries, 0);
    assert_eq!(port.prompts[2].step_index, 1);
    assert_eq!(port.prompts[2].remaining_retries, 1);

    assert_eq!(port.feedback.len(), 3);
    assert_eq!(port.feedback[0].result, AttemptResult::Pending);
    assert_eq!(port.feedback[0].remaining_retries, 1);
    assert_eq!(port.feedback[1].result, AttemptResult::Correct);
    assert_eq!(port.feedback[2].result, AttemptResult::Correct);
    assert_eq!(port.summary.as_ref(), Some(summary));
}

#[test]
fn failure_after_retry_is_captured_in_summary_and_feedback() {
    let mut engine = QuizEngine::from_pgn("1. e4 *", 1).expect("PGN should parse");
    let mut port = DeterministicPort::new(["d4", "Nc3"]);

    let summary = engine.run(&mut port).expect("engine should complete");

    assert_eq!(summary.correct_answers, 0);
    assert_eq!(summary.incorrect_answers, 1);
    assert_eq!(summary.retries_consumed, 1);
    assert_eq!(summary.completed_steps, 1);

    assert_eq!(port.feedback.len(), 2);
    assert_eq!(port.feedback[0].result, AttemptResult::Pending);
    assert_eq!(port.feedback[1].result, AttemptResult::Incorrect);
    assert_eq!(port.feedback[1].solution_san, "e4");
    assert_eq!(
        port.feedback[1].learner_response.as_deref(),
        Some("Nc3")
    );
    assert_eq!(port.summary.as_ref(), Some(summary));
}

#[test]
fn pgn_variations_are_rejected_during_engine_construction() {
    let result = QuizEngine::from_pgn("1. e4 e5 (1... c5) 2. Nf3 Nc6 *", 1);

    assert!(matches!(result, Err(QuizError::VariationsUnsupported)));
}

#[test]
fn empty_pgn_is_rejected_during_engine_construction() {
    let result = QuizEngine::from_pgn("", 1);

    assert!(matches!(result, Err(QuizError::NoMoves)));
}
