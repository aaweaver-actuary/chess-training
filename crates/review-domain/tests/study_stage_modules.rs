use review_domain::study_stage::{self, queries};

#[test]
fn exposes_conversion_function_on_stage_type() {
    assert_eq!(
        study_stage::StudyStage::from_char('n'),
        Some(study_stage::StudyStage::New)
    );
    assert_eq!(
        study_stage::StudyStage::from_char('R'),
        Some(study_stage::StudyStage::Review)
    );
    assert_eq!(study_stage::StudyStage::from_char('x'), None);
}

#[test]
fn exposes_query_helpers_in_submodule() {
    assert!(queries::is_learning(study_stage::StudyStage::Learning));
    assert!(queries::is_active(study_stage::StudyStage::Review));
    assert!(!queries::is_active(study_stage::StudyStage::New));
}

#[test]
fn query_helpers_cover_all_variants() {
    use study_stage::StudyStage;

    assert!(queries::is_new(StudyStage::New));
    assert!(!queries::is_new(StudyStage::Learning));

    assert!(queries::is_learning(StudyStage::Learning));
    assert!(!queries::is_learning(StudyStage::Review));

    assert!(queries::is_review(StudyStage::Review));
    assert!(!queries::is_review(StudyStage::New));

    assert!(queries::is_relearning(StudyStage::Relearning));
    assert!(!queries::is_relearning(StudyStage::Review));

    assert!(queries::is_active(StudyStage::Learning));
    assert!(queries::is_active(StudyStage::Review));
    assert!(queries::is_active(StudyStage::Relearning));
    assert!(!queries::is_active(StudyStage::New));
}
