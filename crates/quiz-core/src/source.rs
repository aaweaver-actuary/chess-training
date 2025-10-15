#![allow(dead_code)]

use shakmaty::san::San;
use shakmaty::{Chess, Position};

use crate::errors::QuizError;

/// Represents a parsed PGN quiz source comprised of a single game's main line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuizSource {
    /// Starting board position prior to the first move.
    pub initial_position: Chess,
    /// Ordered SAN moves that make up the quiz prompts.
    pub san_moves: Vec<San>,
}

impl QuizSource {
    /// Attempts to parse the provided PGN string into a quiz source.
    ///
    /// # Examples
    /// ```rust
    /// use quiz_core::{QuizError, QuizSource};
    /// let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 *";
    /// let source = QuizSource::from_pgn(pgn).expect("valid PGN should parse");
    /// assert_eq!(source.san_moves.len(), 6);
    /// assert_eq!(format!("{}", source.san_moves[0]), "e4");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`QuizError`] when the input includes multiple games, nested
    /// variations, unsupported annotations, or SAN tokens that cannot be
    /// converted into legal moves.
    pub fn from_pgn(pgn: &str) -> Result<Self, QuizError> {
        let trimmed = pgn.trim();
        if trimmed.is_empty() {
            return Err(QuizError::NoMoves);
        }

        // Remove PGN headers (lines starting with '[' and ending with ']')
        let moves_section = trimmed
            .lines()
            .filter(|line| !line.trim_start().starts_with('['))
            .collect::<Vec<_>>()
            .join("\n");

        // Remove comments (enclosed in '{...}' or after ';')
        let mut cleaned = String::new();
        let mut in_brace = false;
        for c in moves_section.chars() {
            match c {
                '{' => in_brace = true,
                '}' => in_brace = false,
                ';' => break, // ignore rest of line after ';'
                _ if !in_brace => cleaned.push(c),
                _ => {}
            }
        }
        let cleaned = cleaned.trim();

        if cleaned.contains('(') || cleaned.contains(')') {
            return Err(QuizError::VariationsUnsupported);
        }

        // TODO: Consider using a proper PGN parser library for more robust validation.

        if trimmed.contains('{')
            || trimmed.contains('}')
            || trimmed.contains(';')
            || trimmed.contains('[')
            || trimmed.contains(']')
        {
            return Err(QuizError::WrongFormat);
        }

        let mut board = Chess::default();
        let initial_position = board.clone();
        let mut san_moves = Vec::new();
        let mut finished = false;

        for raw in trimmed.split_whitespace() {
            if raw.is_empty() {
                continue;
            }

            if finished {
                return Err(QuizError::MultipleGames);
            }

            let token = raw.trim();
            if is_result_token(token) {
                finished = true;
                continue;
            }

            let Some(cleaned) = sanitize_token(token) else {
                continue;
            };

            if cleaned.is_empty() {
                continue;
            }

            let san = San::from_ascii(cleaned.as_bytes())
                .map_err(|_| QuizError::UnreadablePgn(cleaned.clone()))?;
            let mv = san
                .to_move(&board)
                .map_err(|_| QuizError::UnreadablePgn(cleaned.clone()))?;
            board.play_unchecked(mv);
            san_moves.push(san);
        }

        if san_moves.is_empty() {
            return Err(QuizError::NoMoves);
        }

        Ok(Self {
            initial_position,
            san_moves,
        })
    }
}

fn sanitize_token(raw: &str) -> Option<String> {
    let stripped = raw
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.')
        .trim();

    if stripped.is_empty() {
        return None;
    }

    let cleaned = stripped.trim_end_matches(['+', '#', '!', '?']).trim();

    if cleaned.is_empty() {
        return None;
    }

    Some(cleaned.to_string())
}

fn is_result_token(token: &str) -> bool {
    matches!(token, "1-0" | "0-1" | "1/2-1/2" | "*")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_variations() {
        let pgn = "1. e4 e5 (1... c5) 2. Nf3 Nc6 *";
        let err = QuizSource::from_pgn(pgn).unwrap_err();

        assert!(matches!(err, QuizError::VariationsUnsupported));
    }

    #[test]
    fn rejects_multiple_games() {
        let pgn = "1. e4 e5 * 1. d4 d5 *";
        let err = QuizSource::from_pgn(pgn).unwrap_err();

        assert!(matches!(err, QuizError::MultipleGames));
    }

    #[test]
    fn rejects_annotations() {
        let pgn = "1. e4 e5 { comment } 2. Nf3 Nc6 *";
        let err = QuizSource::from_pgn(pgn).unwrap_err();

        assert!(matches!(err, QuizError::WrongFormat));
    }

    #[test]
    fn parses_single_game_main_line() {
        let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 *";
        let source = QuizSource::from_pgn(pgn).expect("single game should parse");

        let moves: Vec<String> = source
            .san_moves
            .iter()
            .map(std::string::ToString::to_string)
            .collect();

        assert_eq!(moves, vec!["e4", "e5", "Nf3", "Nc6", "Bb5", "a6"]);
        assert_eq!(source.initial_position, Chess::default());
    }

    #[test]
    fn rejects_games_without_moves() {
        let err = QuizSource::from_pgn("*").unwrap_err();

        assert!(matches!(err, QuizError::NoMoves));
    }
}
