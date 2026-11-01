// SPDX-License-Identifier: LGPL-3.0-or-later
// GridNode.areTagsEqualIgnoringPlayerId. Values use the caller's tag equality.
use std::collections::BTreeMap;

pub fn same_ignoring_owner<K: Ord, V: PartialEq>(
    new: Option<&BTreeMap<K, V>>,
    old: Option<&BTreeMap<K, V>>,
    owner: &K,
) -> bool {
    for (key, value) in new.into_iter().flat_map(|data| data.iter()) {
        if key != owner && old.and_then(|data| data.get(key)) != Some(value) {
            return false;
        }
    }
    for key in old.into_iter().flat_map(|data| data.keys()) {
        if key != owner && new.is_none_or(|data| !data.contains_key(key)) {
            return false;
        }
    }
    true
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_empty_and_owner_only_are_equal_but_other_key_removal_changes_identity() {
        let mut map = BTreeMap::from([("p", 1)]);
        assert!(same_ignoring_owner(None, Some(&map), &"p"));
        map.insert("e", 20);
        assert!(!same_ignoring_owner(None, Some(&map), &"p"));
        assert!(!same_ignoring_owner(Some(&map), None, &"p"));
        let mut other = map.clone();
        other.insert("p", 99);
        assert!(same_ignoring_owner(Some(&other), Some(&map), &"p"));
        other.insert("e", 30);
        assert!(!same_ignoring_owner(Some(&other), Some(&map), &"p"));
    }
}
