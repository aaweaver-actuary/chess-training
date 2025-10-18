//! PGN tag representation and parsing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagParseError {
    MissingQuotes,
    EmptyKey,
    EmptyValue,
    MissingLeftBracket,
    MissingRightBracket,
}

impl std::fmt::Display for TagParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagParseError::MissingQuotes => write!(f, "missing quotes around tag value"),
            TagParseError::EmptyKey => write!(f, "tag key cannot be empty"),
            TagParseError::EmptyValue => write!(f, "tag value cannot be empty"),
            TagParseError::MissingLeftBracket => write!(f, "missing left bracket '['"),
            TagParseError::MissingRightBracket => write!(f, "missing right bracket ']'"),
        }
    }
}

impl std::error::Error for TagParseError {}

impl TagParseError {
    /// Validates the raw tag string format.
    /// Returns `Ok(())` if valid, or a `TagParseError` if invalid.
    ///
    /// # Examples
    /// ```rust
    /// use chess_training_pgn_import::pgn::TagParseError;
    /// let valid_tag = r#"[Event "World Championship"]"#;
    /// assert!(TagParseError::validate_raw_tag(valid_tag).is_ok());
    ///
    /// let invalid_tag = r#"[Event World Championship]"#; // Missing quotes around World Championship
    /// assert!(TagParseError::validate_raw_tag(invalid_tag).is_err());
    /// ```
    ///
    /// # Errors
    /// Returns a specific `TagParseError` if the tag is invalid.
    pub fn validate_raw_tag(raw: &str) -> Result<(), Self> {
        let raw = raw.trim();
        if !raw.starts_with('[') {
            return Err(TagParseError::MissingLeftBracket);
        }
        if !raw.ends_with(']') {
            return Err(TagParseError::MissingRightBracket);
        }
        let content = &raw[1..raw.len() - 1].trim();
        let mut parts = content.splitn(2, ' ');
        let key = parts.next().ok_or(TagParseError::EmptyKey)?.trim();
        if key.is_empty() {
            return Err(TagParseError::EmptyKey);
        }
        let value = parts.next().ok_or(TagParseError::EmptyValue)?.trim();
        if value.is_empty() {
            return Err(TagParseError::EmptyValue);
        }
        if !value.starts_with('"') || !value.ends_with('"') || value.len() < 2 {
            return Err(TagParseError::MissingQuotes);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PgnTag(String, String);

impl PgnTag {
    #[must_use]
    pub fn new(key: &str, value: &str) -> Self {
        PgnTag(key.to_string(), value.to_string())
    }

    #[must_use]
    pub fn key(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.1
    }
}

impl std::str::FromStr for PgnTag {
    type Err = TagParseError;

    fn from_str(tag: &str) -> Result<Self, Self::Err> {
        // Validate the tag format first and error out if invalid.
        TagParseError::validate_raw_tag(tag)?;

        // Otherwise, we can safely parse it.
        let content_inside_brackets = &tag[1..tag.len() - 1].trim();
        let mut tag_parts = content_inside_brackets.splitn(2, ' ');
        let key = tag_parts.next().ok_or(TagParseError::EmptyKey)?.trim();
        let value = tag_parts.next().ok_or(TagParseError::EmptyValue)?.trim();
        Ok(PgnTag::new(key, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_accessors() {
        let tag = PgnTag::new("Event", "World Championship");
        assert_eq!(tag.key(), "Event");
        assert_eq!(tag.value(), "World Championship");
    }

    #[test]
    fn test_empty_key_and_value() {
        let tag = PgnTag::new("", "");
        assert_eq!(tag.key(), "");
        assert_eq!(tag.value(), "");
    }

    #[test]
    fn test_key_with_whitespace() {
        let tag = PgnTag::new(" Site ", "Moscow");
        assert_eq!(tag.key(), " Site ");
        assert_eq!(tag.value(), "Moscow");
    }

    #[test]
    fn test_value_with_whitespace() {
        let tag = PgnTag::new("Date", " 2023.01.01 ");
        assert_eq!(tag.key(), "Date");
        assert_eq!(tag.value(), " 2023.01.01 ");
    }

    #[test]
    fn test_unicode_key_and_value() {
        let tag = PgnTag::new("Événement", "Чемпионат мира");
        assert_eq!(tag.key(), "Événement");
        assert_eq!(tag.value(), "Чемпионат мира");
    }

    #[test]
    fn test_long_key_and_value() {
        let long_key = "K".repeat(1000);
        let long_value = "V".repeat(1000);
        let tag = PgnTag::new(&long_key, &long_value);
        assert_eq!(tag.key(), long_key);
        assert_eq!(tag.value(), long_value);
    }

    #[test]
    fn test_special_characters() {
        let tag = PgnTag::new("Key!@#$%^&*()", "Value[]{};:'\",.<>/?\\|`~");
        assert_eq!(tag.key(), "Key!@#$%^&*()");
        assert_eq!(tag.value(), "Value[]{};:'\",.<>/?\\|`~");
    }

    #[test]
    fn test_null_bytes_in_key_and_value() {
        let tag = PgnTag::new("Key\0WithNull", "Value\0WithNull");
        assert_eq!(tag.key(), "Key\0WithNull");
        assert_eq!(tag.value(), "Value\0WithNull");
    }
}
