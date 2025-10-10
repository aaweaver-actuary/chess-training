use review_domain::study_stage::{self, conversions, queries};

#[test]
fn exposes_conversion_functions_in_submodule() {
    assert_eq!(
        conversions::from_char('n'),
        Some(study_stage::StudyStage::New)
    );
    assert_eq!(
        conversions::from_char('R'),
        Some(study_stage::StudyStage::Review)
    );
    assert_eq!(conversions::from_char('x'), None);
}

#[test]
fn exposes_query_helpers_in_submodule() {
    assert!(queries::is_learning(study_stage::StudyStage::Learning));
    assert!(queries::is_active(study_stage::StudyStage::Review));
    assert!(!queries::is_active(study_stage::StudyStage::New));
}
