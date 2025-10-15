#![allow(dead_code)]

/// Marker trait describing how adapters will interact with the quiz engine.
pub trait QuizPort {}

/// Placeholder feedback message sent through adapter ports.
#[derive(Debug, Default)]
pub struct FeedbackMessage;
