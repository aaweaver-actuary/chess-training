//! Shared unlock record representation.

use chrono::NaiveDate;

/// Represents a record of new study material being unlocked for a learner.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnlockRecord<Owner, Detail> {
    /// Identifier of the learner receiving the unlock.
    pub owner_id: Owner,
    /// Domain-specific payload describing what was unlocked.
    pub detail: Detail,
    /// Day on which the unlock occurred.
    pub unlocked_on: NaiveDate,
}

impl<Owner, Detail> UnlockRecord<Owner, Detail> {
    /// Maps the domain-specific payload to a different type while preserving metadata.
    pub fn map_detail<D2>(self, mapper: impl FnOnce(Detail) -> D2) -> UnlockRecord<Owner, D2> {
        UnlockRecord {
            owner_id: self.owner_id,
            detail: mapper(self.detail),
            unlocked_on: self.unlocked_on,
        }
    }
}
