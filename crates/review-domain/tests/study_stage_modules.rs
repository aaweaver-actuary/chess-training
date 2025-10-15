use review_domain::study_stage;

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
