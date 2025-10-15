#![allow(dead_code)]

use chess_training_pgn_import::parse_games;
use shakmaty::san::{ParseSanError, San, SanError};
use shakmaty::{Chess, Position};

use crate::errors::{QuizError, QuizResult};

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
    pub fn from_pgn(pgn: &str) -> QuizResult<Self> {
        let trimmed = pgn.trim();
        if trimmed.is_empty() {
            return Err(QuizError::NoMoves);
        }

        let mut games = parse_games(trimmed);

        if games.is_empty() {
            return Err(QuizError::NoMoves);
        }

        if games.len() > 1 {
            return Err(QuizError::MultipleGames);
        }

        let game = games.remove(0);

        if game.saw_variation_markers {
            return Err(QuizError::VariationsUnsupported);
        }

        if game.saw_comment_markers {
            return Err(QuizError::WrongFormat);
        }

        if game.tokens_after_result {
            return Err(QuizError::MultipleGames);
        }

        if game.moves.is_empty() {
            return Err(QuizError::NoMoves);
        }

        let mut board = Chess::default();
        let initial_position = board.clone();
        let mut san_moves = Vec::new();
        for cleaned in game.moves {
            let san = San::from_ascii(cleaned.as_bytes()).map_err(|err: ParseSanError| {
                QuizError::unreadable_from_parse(cleaned.clone(), err)
            })?;
            let mv = san
                .to_move(&board)
                .map_err(|err: SanError| QuizError::unreadable_from_san(cleaned.clone(), err))?;
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
    fn rejects_line_comments() {
        let pgn = "1. e4 e5 ; sideline 2. Nf3 Nc6 *";
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
    fn parses_moves_after_normalisation() {
        let pgn = "1. e4! e5?! 2. Nf3+ Nc6# 3. Bb5!! a6?? *";
        let source = QuizSource::from_pgn(pgn).expect("annotated moves should normalise");

        let moves: Vec<String> = source
            .san_moves
            .iter()
            .map(std::string::ToString::to_string)
            .collect();

        assert_eq!(moves, vec!["e4", "e5", "Nf3", "Nc6", "Bb5", "a6"],);
    }

    #[test]
    fn rejects_games_without_moves() {
        let err = QuizSource::from_pgn("*").unwrap_err();

        assert!(matches!(err, QuizError::NoMoves));
    }
}
