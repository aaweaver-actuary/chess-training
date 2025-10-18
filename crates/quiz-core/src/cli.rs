use std::io::{self, BufRead, BufReader, Write};

use crate::errors::AdapterResult;
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
    fn present_prompt(&mut self, context: PromptContext) -> AdapterResult<String> {
        writeln!(
            self.writer,
            "\nMove {}/{}",
            context.display_index(),
            context.total_steps
        )?;
        writeln!(self.writer, "Board FEN: {}", context.board_fen)?;

        if let Some(previous) = context.previous_move_san.as_deref() {
            writeln!(self.writer, "Previous move: {previous}")?;
        }

        if let Some(step_id) = context.metadata.step_id.as_deref() {
            writeln!(self.writer, "Step ID: {step_id}")?;
        }

        if !context.metadata.theme_tags.is_empty() {
            writeln!(
                self.writer,
                "Themes: {}",
                context.metadata.theme_tags.join(", ")
            )?;
        }

        if !context.metadata.card_ids.is_empty() {
            writeln!(
                self.writer,
                "Card references: {}",
                context.metadata.card_ids.join(", ")
            )?;
        }

        writeln!(self.writer, "Your move (SAN): {}", context.prompt_san)?;

        if context.remaining_retries > 0 {
            writeln!(
                self.writer,
                "Retries remaining after this attempt: {}",
                context.remaining_retries
            )?;
        }

        write!(self.writer, "> ")?;
        self.writer.flush()?;

        let mut buffer = String::new();
        self.reader.read_line(&mut buffer)?;

        Ok(buffer.trim().to_string())
    }

    fn publish_feedback(&mut self, feedback: FeedbackMessage) -> AdapterResult<()> {
        match feedback.result {
            AttemptResult::Correct => {
                writeln!(self.writer, "Correct!")?;
                for note in &feedback.annotations {
                    writeln!(self.writer, "Note: {note}")?;
                }
            }
            AttemptResult::Pending => {
                writeln!(self.writer, "Incorrect, try again.")?;
                writeln!(
                    self.writer,
                    "Retries remaining: {}",
                    feedback.remaining_retries
                )?;

                if let Some(response) = &feedback.learner_response {
                    writeln!(self.writer, "Your answer: {response}")?;
                }
            }
            AttemptResult::Incorrect => {
                writeln!(self.writer, "Incorrect.")?;

                if let Some(response) = &feedback.learner_response {
                    writeln!(self.writer, "Your answer: {response}")?;
                }

                if !feedback.solution_san.is_empty() {
                    writeln!(self.writer, "Solution: {}", feedback.solution_san)?;
                }

                if !feedback.annotations.is_empty() {
                    writeln!(self.writer, "Annotations:")?;
                    for note in &feedback.annotations {
                        writeln!(self.writer, "- {note}")?;
                    }
                }
            }
        }

        self.writer.flush()?;
        Ok(())
    }

    fn present_summary(&mut self, summary: &QuizSummary) -> AdapterResult<()> {
        writeln!(
            self.writer,
            "\nQuiz complete: {}/{} steps",
            summary.completed_steps, summary.total_steps
        )?;
        writeln!(self.writer, "Correct: {}", summary.correct_answers)?;
        writeln!(self.writer, "Incorrect: {}", summary.incorrect_answers)?;
        writeln!(self.writer, "Retries used: {}", summary.retries_consumed)?;
        self.writer.flush()?;
        Ok(())
    }
}

/// Placeholder CLI adapter entry point for manual smoke tests.
pub fn run() {
    eprintln!("quiz-core CLI adapter is not yet orchestrating a session");
}
