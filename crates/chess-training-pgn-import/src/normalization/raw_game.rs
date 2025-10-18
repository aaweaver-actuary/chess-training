//! Raw representation of a parsed PGN game, before normalization.

/// A raw representation of a parsed PGN game, before any normalization has been applied.
/// Includes tags, move tokens, and flags indicating the presence of variations or comments.
///
/// # Examples
/// ```rust
/// use chess_training_pgn_import::normalization::RawGame;
/// let game = RawGame {
///     tags: vec![("Event".to_string(), "My Game".to_string())],
///     moves: vec!["e4".to_string(), "e5".to_string()],
///     saw_variation_markers: false,
///     saw_comment_markers: false,
///     saw_result_token: true,
///     tokens_after_result: false,
/// };
/// assert_eq!(game.tag("Event"), Some("My Game"));
/// assert!(game.has_content());
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RawGame {
    pub tags: Vec<(String, String)>,
    pub moves: Vec<String>,
    pub saw_variation_markers: bool,
    pub saw_comment_markers: bool,
    pub saw_result_token: bool,
    pub tokens_after_result: bool,
}

/// Builder for constructing a `RawGame` instance incrementally.
/// Allows adding tags and moves one at a time, as well as setting flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawGameBuilder {
    pub tags: Option<Vec<(String, String)>>,
    pub moves: Option<Vec<String>>,
    pub saw_variation_markers: Option<bool>,
    pub saw_comment_markers: Option<bool>,
    pub saw_result_token: Option<bool>,
    pub tokens_after_result: Option<bool>,
}

impl RawGame {
    /// Returns an instance of the builder for constructing a `RawGame`.
    pub fn builder() -> RawGameBuilder {
        RawGameBuilder::default()
    }

    /// Retrieves the value of a tag by name, case-insensitively.
    /// Returns `None` if the tag is not present.
    pub fn tag(&self, name: &str) -> Option<&str> {
        self.tags
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// Returns true if the game has any tags.
    pub fn has_tags(&self) -> bool {
        !self.tags.is_empty()
    }

    /// Returns true if the game has any moves.
    pub fn has_moves(&self) -> bool {
        !self.moves.is_empty()
    }

    /// Returns true if the game has any tags or moves.
    pub fn has_content(&self) -> bool {
        self.has_tags() || self.has_moves()
    }
}

impl RawGameBuilder {
    /// Adds a tag key-value pair to the game.
    pub fn add_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into();
        let value = value.into();
        if let Some(tags) = &mut self.tags {
            tags.push((key, value));
        } else {
            self.tags = Some(vec![(key, value)]);
        }
        self
    }

    /// Adds a ply (move token) to the game's move list.
    pub fn add_ply(mut self, token: impl Into<String>) -> Self {
        let token = token.into();
        if let Some(moves) = &mut self.moves {
            moves.push(token);
        } else {
            self.moves = Some(vec![token]);
        }
        self
    }

    /// Sets the `saw_variation_markers` flag to `true`.
    /// Since the default is `false`, this method does not need a parameter.
    pub fn has_variation_markers(mut self) -> Self {
        self.saw_variation_markers = Some(true);
        self
    }

    /// Sets the `saw_comment_markers` flag to `true`.
    /// Since the default is `false`, this method does not need a parameter.
    pub fn has_comment_markers(mut self) -> Self {
        self.saw_comment_markers = Some(true);
        self
    }

    /// Sets the `saw_result_token` flag to `true`.
    /// Since the default is `false`, this method does not need a parameter.
    pub fn has_result_token(mut self) -> Self {
        self.saw_result_token = Some(true);
        self
    }

    /// Sets the `tokens_after_result` flag to `true`.
    /// Since the default is `false`, this method does not need a parameter.
    pub fn has_tokens_after_result(mut self) -> Self {
        self.tokens_after_result = Some(true);
        self
    }

    /// Builds the `RawGame` instance, consuming the builder.
    /// Any fields not explicitly set will use their default values.
    pub fn build(self) -> Result<RawGame, &'static str> {
        Ok(RawGame {
            tags: self.tags.unwrap_or_default(),
            moves: self.moves.unwrap_or_default(),
            saw_variation_markers: self.saw_variation_markers.unwrap_or(false),
            saw_comment_markers: self.saw_comment_markers.unwrap_or(false),
            saw_result_token: self.saw_result_token.unwrap_or(false),
            tokens_after_result: self.tokens_after_result.unwrap_or(false),
        })
    }
}

impl Default for RawGameBuilder {
    /// Provides default values for the builder fields.
    /// By default, tags and moves are empty vectors, and all flags are false.
    fn default() -> Self {
        RawGameBuilder {
            tags: None,
            moves: None,
            saw_variation_markers: Some(false),
            saw_comment_markers: Some(false),
            saw_result_token: Some(false),
            tokens_after_result: Some(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_game() {
        let game = RawGameBuilder::default()
            .add_tag("Event", "My Game")
            .add_ply("e4")
            .add_ply("e5")
            .has_result_token()
            .build()
            .expect("Failed to build RawGame");

        assert_eq!(game.tag("Event"), Some("My Game"));
        assert!(game.has_content());
    }

    #[test]
    fn test_can_retrieve_tags_by_name() {
        let game = RawGame::builder()
            .add_tag("Event", "My Game")
            .add_tag("White", "Alice")
            .add_tag("Black", "Bob")
            .build()
            .unwrap();
        assert_eq!(game.tag("Event"), Some("My Game"));
        assert_eq!(game.tag("White"), Some("Alice"));
        assert_eq!(game.tag("Black"), Some("Bob"));
        assert_eq!(game.tag("Unknown"), None);
    }

    #[test]
    fn test_can_retrieve_tags_case_insensitively() {
        let game = RawGame::builder()
            .add_tag("Event", "My Game")
            .build()
            .unwrap();
        assert_eq!(game.tag("event"), Some("My Game"));
        assert_eq!(game.tag("EVENT"), Some("My Game"));
        assert_eq!(game.tag("EvEnT"), Some("My Game"));
    }

    #[test]
    fn test_has_tags() {
        let game_with_tags = RawGame::builder()
            .add_tag("Event", "My Game")
            .build()
            .unwrap();
        assert!(game_with_tags.has_tags());

        let game_without_tags = RawGame::builder().build().unwrap();
        assert!(!game_without_tags.has_tags());
    }

    #[test]
    fn test_has_moves() {
        let game_with_moves = RawGame::builder()
            .add_ply("e4")
            .add_ply("e5")
            .build()
            .unwrap();
        assert!(game_with_moves.has_moves());

        let game_without_moves = RawGame::builder().build().unwrap();
        assert!(!game_without_moves.has_moves());
    }

    #[test]
    fn test_has_content() {
        let game_with_tags_no_moves = RawGame::builder()
            .add_tag("Event", "My Game")
            .build()
            .unwrap();
        assert!(game_with_tags_no_moves.has_tags());
        assert!(!game_with_tags_no_moves.has_moves());
        assert!(game_with_tags_no_moves.has_content());

        let game_with_moves_no_tags = RawGame::builder()
            .add_ply("e4")
            .add_ply("e5")
            .build()
            .unwrap();
        assert!(!game_with_moves_no_tags.has_tags());
        assert!(game_with_moves_no_tags.has_moves());
        assert!(game_with_moves_no_tags.has_content());

        let game_with_both = RawGame::builder()
            .add_tag("Event", "My Game")
            .add_ply("e4")
            .add_ply("e5")
            .build()
            .unwrap();
        assert!(game_with_both.has_tags());
        assert!(game_with_both.has_moves());
        assert!(game_with_both.has_content());
    }

    #[test]
    fn test_builder_defaults() {
        let game = RawGame::builder().build().unwrap();
        assert!(!game.has_tags());
        assert!(!game.has_moves());
        assert!(!game.saw_variation_markers);
        assert!(!game.saw_comment_markers);
        assert!(!game.saw_result_token);
        assert!(!game.tokens_after_result);
    }

    #[test]
    fn test_builder_add_tag() {
        let game = RawGame::builder()
            .add_tag("Event", "My Game")
            .add_tag("White", "Alice")
            .build()
            .unwrap();
        assert_eq!(game.tags.len(), 2);
        assert_eq!(game.tag("Event"), Some("My Game"));
        assert_eq!(game.tag("White"), Some("Alice"));
        assert!(game.has_tags());
    }

    #[test]
    fn test_builder_add_ply() {
        let game = RawGame::builder()
            .add_ply("e4")
            .add_ply("e5")
            .build()
            .unwrap();
        assert_eq!(game.moves.len(), 2);
        assert_eq!(game.moves[0], "e4");
        assert_eq!(game.moves[1], "e5");
        assert!(game.has_moves());
    }

    #[test]
    fn test_builder_flags() {
        let game_w_variation_markers = RawGame::builder()
            .add_ply("e4")
            .has_variation_markers()
            .build()
            .unwrap();
        assert!(game_w_variation_markers.saw_variation_markers);

        let game_w_comment_markers = RawGame::builder()
            .add_ply("e4")
            .has_comment_markers()
            .build()
            .unwrap();
        assert!(game_w_comment_markers.saw_comment_markers);

        let game_w_result_token = RawGame::builder()
            .add_ply("e4")
            .has_result_token()
            .build()
            .unwrap();
        assert!(game_w_result_token.saw_result_token);

        let game_w_tokens_after_result = RawGame::builder()
            .add_ply("e4")
            .has_tokens_after_result()
            .build()
            .unwrap();
        assert!(game_w_tokens_after_result.tokens_after_result);
    }
}
