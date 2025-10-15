use std::io::{self, BufRead, BufReader, Write};

use crate::errors::QuizError;
use crate::ports::{FeedbackMessage, PromptContext, QuizPort};
use crate::state::{AttemptResult, QuizSummary};

/// Terminal-backed adapter implementing the [`QuizPort`] contract.
pub struct TerminalPort<R, W> {
    reader: R,
    writer: W,
}

impl TerminalPort<BufReader<io::Stdin>, io::Stdout> {
    /// Constructs a terminal port using standard input and output streams.
    #[must_use]
    pub fn new() -> Self {
        Self::with_io(BufReader::new(io::stdin()), io::stdout())
    }
}

impl Default for TerminalPort<BufReader<io::Stdin>, io::Stdout> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R, W> TerminalPort<R, W> {
    /// Creates a terminal port from custom reader and writer handles.
    #[must_use]
    pub fn with_io(reader: R, writer: W) -> Self {
        Self { reader, writer }
    }

    /// Consumes the port and returns the underlying I/O handles.
    #[must_use]
    pub fn into_inner(self) -> (R, W) {
        (self.reader, self.writer)
    }
}

impl<R, W> QuizPort for TerminalPort<R, W>
where
    R: BufRead,
    W: Write,
{
    fn present_prompt(&mut self, context: PromptContext) -> Result<String, QuizError> {
        writeln!(
            self.writer,
            "\nMove {}/{}",
            context.display_index(),
            context.total_steps
        )
        .map_err(|_| QuizError::Io)?;
        writeln!(self.writer, "Board FEN: {}", context.board_fen).map_err(|_| QuizError::Io)?;

        if let Some(previous) = context.previous_move_san.as_deref() {
            writeln!(self.writer, "Previous move: {previous}").map_err(|_| QuizError::Io)?;
        }

        writeln!(self.writer, "Your move (SAN): {}", context.prompt_san)
            .map_err(|_| QuizError::Io)?;

        if context.remaining_retries > 0 {
            writeln!(
                self.writer,
                "Retries remaining after this attempt: {}",
                context.remaining_retries
            )
            .map_err(|_| QuizError::Io)?;
        }

        write!(self.writer, "> ").map_err(|_| QuizError::Io)?;
        self.writer.flush().map_err(|_| QuizError::Io)?;

        let mut buffer = String::new();
        self.reader
            .read_line(&mut buffer)
            .map_err(|_| QuizError::Io)?;

        Ok(buffer.trim().to_string())
    }

    fn publish_feedback(&mut self, feedback: FeedbackMessage) -> Result<(), QuizError> {
        match feedback.result {
            AttemptResult::Correct => {
                writeln!(self.writer, "Correct!").map_err(|_| QuizError::Io)?;
                for note in &feedback.annotations {
                    writeln!(self.writer, "Note: {note}").map_err(|_| QuizError::Io)?;
                }
            }
            AttemptResult::Pending => {
                writeln!(self.writer, "Incorrect, try again.").map_err(|_| QuizError::Io)?;
                writeln!(
                    self.writer,
                    "Retries remaining: {}",
                    feedback.remaining_retries
                )
                .map_err(|_| QuizError::Io)?;

                if let Some(response) = &feedback.learner_response {
                    writeln!(self.writer, "Your answer: {response}").map_err(|_| QuizError::Io)?;
                }
            }
            AttemptResult::Incorrect => {
                writeln!(self.writer, "Incorrect.").map_err(|_| QuizError::Io)?;

                if let Some(response) = &feedback.learner_response {
                    writeln!(self.writer, "Your answer: {response}").map_err(|_| QuizError::Io)?;
                }

                if !feedback.solution_san.is_empty() {
                    writeln!(self.writer, "Solution: {}", feedback.solution_san)
                        .map_err(|_| QuizError::Io)?;
                }

                if !feedback.annotations.is_empty() {
                    writeln!(self.writer, "Annotations:").map_err(|_| QuizError::Io)?;
                    for note in &feedback.annotations {
                        writeln!(self.writer, "- {note}").map_err(|_| QuizError::Io)?;
                    }
                }
            }
        }

        self.writer.flush().map_err(|_| QuizError::Io)
    }

    fn present_summary(&mut self, summary: &QuizSummary) -> Result<(), QuizError> {
        writeln!(
            self.writer,
            "\nQuiz complete: {}/{} steps",
            summary.completed_steps, summary.total_steps
        )
        .map_err(|_| QuizError::Io)?;
        writeln!(self.writer, "Correct: {}", summary.correct_answers).map_err(|_| QuizError::Io)?;
        writeln!(self.writer, "Incorrect: {}", summary.incorrect_answers)
            .map_err(|_| QuizError::Io)?;
        writeln!(self.writer, "Retries used: {}", summary.retries_consumed)
            .map_err(|_| QuizError::Io)?;
        self.writer.flush().map_err(|_| QuizError::Io)
    }
}

/// Placeholder CLI adapter entry point for manual smoke tests.
pub fn run() {
    eprintln!("quiz-core CLI adapter is not yet orchestrating a session");
}
