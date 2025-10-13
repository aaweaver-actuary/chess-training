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
    #[must_use]
    pub fn map_detail<D2>(self, mapper: impl FnOnce(Detail) -> D2) -> UnlockRecord<Owner, D2> {
        UnlockRecord {
            owner_id: self.owner_id,
            detail: mapper(self.detail),
            unlocked_on: self.unlocked_on,
        }
    }
}

use crate::ids::EdgeId;

/// Domain payload stored for each unlock record.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnlockDetail {
    /// Identifier of the unlocked opening edge.
    pub edge_id: EdgeId,
}

impl UnlockDetail {
    /// Creates a new unlock detail payload.
    #[must_use]
    pub fn new(edge_id: EdgeId) -> Self {
        Self { edge_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::EdgeId;

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    #[test]
    fn unlock_record_map_detail_transforms_payload() {
        let record = UnlockRecord {
            owner_id: "owner",
            detail: UnlockDetail::new(EdgeId::new(7)),
            unlocked_on: naive_date(2023, 1, 1),
        };
        let mapped = record.map_detail(|detail| detail.edge_id.get() + 1);
        assert_eq!(mapped.detail, 8);
        assert_eq!(mapped.owner_id, "owner");
        assert_eq!(mapped.unlocked_on, naive_date(2023, 1, 1));
    }

    #[test]
    fn unlock_detail_constructor_sets_edge_id() {
        assert_eq!(UnlockDetail::new(EdgeId::new(99)).edge_id, EdgeId::new(99));
    }
}
