#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NormalizedLine {
    pub tokens: Vec<String>,
    pub saw_variation_markers: bool,
    pub saw_comment_markers: bool,
    pub saw_result_token: bool,
    pub tokens_after_result: bool,
}
