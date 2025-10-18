pub mod normalized_line;
pub mod raw_game;

pub use normalized_line::NormalizedLine;
pub use raw_game::RawGame;

/// Parses the input PGN string into a vector of `RawGame` instances.
/// Each `RawGame` contains the tags and moves extracted from the PGN.
pub fn parse_games(input: &str) -> Vec<RawGame> {
    let mut games = Vec::new();
    let mut current = RawGame::default();
    let mut header_in_progress = false;
    let mut saw_moves = false;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('[') {
            if !header_in_progress && current.has_content() {
                games.push(current);
                current = RawGame::default();
                saw_moves = false;
            }
            header_in_progress = true;
            if let Some(tag) = parse_tag(trimmed) {
                current.tags.push(tag);
            }
            continue;
        }

        header_in_progress = false;
        saw_moves = true;
        let normalized = normalize_line(trimmed);
        current.moves.extend(normalized.tokens);
        if normalized.saw_variation_markers {
            current.saw_variation_markers = true;
        }
        if normalized.saw_comment_markers {
            current.saw_comment_markers = true;
        }
        if normalized.saw_result_token {
            current.saw_result_token = true;
        }
        if normalized.tokens_after_result {
            current.tokens_after_result = true;
        }
    }

    if saw_moves || current.has_content() {
        games.push(current);
    }

    games
}

pub fn normalize_line(line: &str) -> NormalizedLine {
    let mut tokens = Vec::new();
    let mut saw_variation_markers = false;
    let mut saw_comment_markers = false;
    let mut saw_result_token = false;
    let mut tokens_after_result = false;
    let mut in_brace_comment = false;
    let mut after_result = false;

    for raw in line.split_whitespace() {
        if raw.is_empty() {
            continue;
        }

        if after_result {
            tokens_after_result = true;
        }

        if in_brace_comment {
            if raw.contains('}') {
                saw_comment_markers = true;
                in_brace_comment = false;
            }
            continue;
        }

        if raw.starts_with(';') {
            saw_comment_markers = true;
            break;
        }

        if raw.contains('{') {
            saw_comment_markers = true;
            if !raw.contains('}') {
                in_brace_comment = true;
            }
            continue;
        }

        if raw.contains('}') {
            saw_comment_markers = true;
            continue;
        }

        if raw.contains('(') || raw.contains(')') {
            saw_variation_markers = true;
            continue;
        }

        if is_result_token(raw) {
            saw_result_token = true;
            after_result = true;
            continue;
        }

        if after_result {
            continue;
        }

        if let Some(token) = sanitize_token(raw) {
            tokens.push(token);
        }
    }

    NormalizedLine {
        tokens,
        saw_variation_markers,
        saw_comment_markers,
        saw_result_token,
        tokens_after_result,
    }
}

pub fn sanitize_token(raw: &str) -> Option<String> {
    if is_result_token(raw) {
        return None;
    }

    if raw.contains('{') || raw.contains('}') || raw.contains('(') || raw.contains(')') {
        return None;
    }

    let stripped = raw
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.')
        .trim();

    if stripped.is_empty() {
        return None;
    }

    let cleaned = stripped.trim_end_matches(['!', '?', '+', '#']).trim();

    if cleaned.is_empty() {
        return None;
    }

    Some(cleaned.to_string())
}

fn is_result_token(token: &str) -> bool {
    matches!(token, "1-0" | "0-1" | "1/2-1/2" | "*")
}

#[must_use]
pub fn parse_tag(line: &str) -> Option<(String, String)> {
    let trimmed = line.strip_prefix('[').and_then(|s| s.strip_suffix(']'))?;
    let (key, raw_value) = trimmed.split_once(' ')?;
    let value = raw_value.trim().strip_prefix('"')?.strip_suffix('"')?;
    Some((key.to_string(), value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_token_removes_results_and_markers() {
        assert_eq!(sanitize_token("1-0"), None);
        assert_eq!(sanitize_token("{comment}"), None);
        assert_eq!(sanitize_token("(variation)"), None);
    }

    #[test]
    fn sanitize_token_strips_move_numbers_and_glyphs() {
        assert_eq!(sanitize_token("12...Qxe4+!?"), Some("Qxe4".to_string()));
    }

    #[test]
    fn sanitize_token_drops_tokens_without_moves() {
        assert_eq!(sanitize_token("12...?!"), None);
    }

    #[test]
    fn parse_tag_reads_key_value_pairs() {
        let tag = parse_tag("[Event \"Test\"]");
        assert_eq!(tag, Some(("Event".into(), "Test".into())));
        assert_eq!(parse_tag("[Event Test]"), None);
        assert_eq!(parse_tag("[Event \"Test]"), None);
    }

    #[test]
    fn normalize_line_detects_comments_and_variations() {
        let line = "1. e4 {comment} (sideline) e5";
        let normalized = normalize_line(line);
        assert!(normalized.saw_comment_markers);
        assert!(normalized.saw_variation_markers);
        assert_eq!(normalized.tokens, vec!["e4".to_string(), "e5".to_string()]);
    }

    #[test]
    fn normalize_line_tracks_trailing_content_after_result() {
        let line = "1. e4 e5 * 1. d4";
        let normalized = normalize_line(line);
        assert!(normalized.saw_result_token);
        assert!(normalized.tokens_after_result);
        assert_eq!(normalized.tokens, vec!["e4".to_string(), "e5".to_string()]);
    }

    #[test]
    fn parse_games_preserves_tags_and_moves() {
        let pgn = "[Event \"Game\"]\n\n1. e4 e5\n\n[Event \"Second\"]\n1. d4 d5 *";
        let games = parse_games(pgn);
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].moves, vec!["e4".to_string(), "e5".to_string()]);
        assert_eq!(games[1].moves, vec!["d4".to_string(), "d5".to_string()]);
    }

    #[test]
    fn parse_games_handles_headers_without_moves() {
        let pgn = "[Event \"Header Only\"]";
        let games = parse_games(pgn);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].tags.len(), 1);
        assert!(games[0].moves.is_empty());
    }

    #[test]
    fn parse_games_skips_malformed_headers_but_keeps_moves() {
        let pgn = "[Malformed\n1. e4 e5 *";
        let games = parse_games(pgn);
        assert_eq!(games.len(), 1);
        assert!(games[0].tags.is_empty());
        let moves = &games[0].moves;
        assert_eq!(moves.len(), 2);
        assert_eq!(moves[0], "e4");
        assert_eq!(moves[1], "e5");
    }

    #[test]
    fn parse_games_ignores_empty_input() {
        assert!(parse_games("").is_empty());
        assert!(parse_games(" \n\n\t  ").is_empty());
    }
}
