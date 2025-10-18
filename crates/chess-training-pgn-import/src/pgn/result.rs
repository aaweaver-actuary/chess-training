use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PgnResult {
    WhiteWin,
    BlackWin,
    Draw,
    Unfinished,
}

impl PgnResult {
    /// Returns the string slice representation of the PGN result.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            PgnResult::WhiteWin => "1-0",
            PgnResult::BlackWin => "0-1",
            PgnResult::Draw => "1/2-1/2",
            PgnResult::Unfinished => "*",
        }
    }

    /// Returns `true` if the game is finished (i.e., not unfinished).
    /// This is a convenience method.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        *self != PgnResult::Unfinished
    }
}

impl FromStr for PgnResult {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1-0" => Ok(PgnResult::WhiteWin),
            "0-1" => Ok(PgnResult::BlackWin),
            "1/2-1/2" => Ok(PgnResult::Draw),
            "*" => Ok(PgnResult::Unfinished),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for PgnResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_parses_valid_results() {
        assert_eq!(PgnResult::from_str("1-0"), Ok(PgnResult::WhiteWin));
        assert_eq!(PgnResult::from_str("0-1"), Ok(PgnResult::BlackWin));
        assert_eq!(PgnResult::from_str("1/2-1/2"), Ok(PgnResult::Draw));
        assert_eq!(PgnResult::from_str("*"), Ok(PgnResult::Unfinished));
    }

    #[test]
    fn from_str_rejects_invalid_results() {
        assert_eq!(PgnResult::from_str("invalid"), Err(()));
        assert_eq!(PgnResult::from_str(""), Err(()));
        assert_eq!(PgnResult::from_str("1-0 "), Err(()));
        assert_eq!(PgnResult::from_str(" 1-0"), Err(()));
        assert_eq!(PgnResult::from_str("1/2-1/2 "), Err(()));
        assert_eq!(PgnResult::from_str("1/2-1/3"), Err(()));
        assert_eq!(PgnResult::from_str("0-0"), Err(()));
        assert_eq!(PgnResult::from_str("1-1"), Err(()));
        assert_eq!(PgnResult::from_str("draw"), Err(()));
        assert_eq!(PgnResult::from_str("WHITEWIN"), Err(()));
        assert_eq!(PgnResult::from_str("0-1-0"), Err(()));
        assert_eq!(PgnResult::from_str("**"), Err(()));
    }

    #[test]
    fn as_str_returns_expected_strings() {
        assert_eq!(PgnResult::WhiteWin.as_str(), "1-0");
        assert_eq!(PgnResult::BlackWin.as_str(), "0-1");
        assert_eq!(PgnResult::Draw.as_str(), "1/2-1/2");
        assert_eq!(PgnResult::Unfinished.as_str(), "*");
    }

    #[test]
    fn display_trait_outputs_expected_strings() {
        assert_eq!(PgnResult::WhiteWin.to_string(), "1-0");
        assert_eq!(PgnResult::BlackWin.to_string(), "0-1");
        assert_eq!(PgnResult::Draw.to_string(), "1/2-1/2");
        assert_eq!(PgnResult::Unfinished.to_string(), "*");
    }

    #[test]
    fn is_finished_returns_true_for_finished_games() {
        assert!(PgnResult::WhiteWin.is_finished());
        assert!(PgnResult::BlackWin.is_finished());
        assert!(PgnResult::Draw.is_finished());
    }

    #[test]
    fn is_finished_returns_false_for_unfinished_game() {
        assert!(!PgnResult::Unfinished.is_finished());
    }

    #[test]
    fn enum_equality_and_copy() {
        let a = PgnResult::WhiteWin;
        let b = a;
        assert_eq!(a, b);
        let c = PgnResult::BlackWin;
        assert_ne!(a, c);
    }

    #[test]
    fn clone_trait_works() {
        let a = PgnResult::Draw;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn debug_trait_outputs_expected_format() {
        let s = format!("{:?}", PgnResult::Draw);
        assert_eq!(s, "Draw");
    }
}
