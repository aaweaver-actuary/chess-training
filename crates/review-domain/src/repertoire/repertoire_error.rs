/// Domain error produced when manipulating a [`Repertoire`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RepertoireError {
    /// Placeholder error returned by not-yet-implemented operations.
    #[error("repertoire operation '{operation}' is not implemented yet")]
    NotImplemented { operation: &'static str },
}

impl RepertoireError {
    /// Creates a [`RepertoireError::NotImplemented`] for the provided operation.
    #[must_use]
    pub const fn not_implemented(operation: &'static str) -> Self {
        Self::NotImplemented { operation }
    }
}

#[cfg(test)]
mod tests {
    use super::RepertoireError;

    #[test]
    fn test_not_implemented_error() {
        let operation = "add_move";
        let error = RepertoireError::not_implemented(operation);
        assert_eq!(error, RepertoireError::NotImplemented { operation });
        assert_eq!(
            format!("{error}"),
            "repertoire operation 'add_move' is not implemented yet"
        );
    }
}
