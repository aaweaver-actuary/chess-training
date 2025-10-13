use std::collections::HashSet;

use crate::model::UnlockRecord;
use crate::store::StoreError;

pub(super) fn insert_unlock_or_error(
    unlocks: &mut HashSet<UnlockRecord>,
    unlock: &UnlockRecord,
) -> Result<(), StoreError> {
    if unlocks.insert(unlock.clone()) {
        Ok(())
    } else {
        Err(StoreError::DuplicateUnlock {
            edge: unlock.detail.edge_id,
            day: unlock.unlocked_on,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EdgeId, UnlockDetail};
    use chrono::NaiveDate;

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    #[test]
    fn insert_unlock_or_error_prevents_duplicates() {
        let mut unlocks = HashSet::new();
        let record = UnlockRecord {
            owner_id: "owner".into(),
            detail: UnlockDetail {
                edge_id: EdgeId::new(7),
            },
            unlocked_on: naive_date(2023, 1, 1),
        };
        insert_unlock_or_error(&mut unlocks, &record).expect("first insert succeeds");
        let err = insert_unlock_or_error(&mut unlocks, &record).unwrap_err();
        assert!(matches!(
            err,
            StoreError::DuplicateUnlock {
                edge,
                ..
            } if edge == EdgeId::new(7)
        ));
    }
}
