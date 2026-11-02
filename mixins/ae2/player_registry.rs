// SPDX-License-Identifier: LGPL-3.0-or-later
// PlayerRegistryInternal's UUID <-> int map, source parallel-array save layout.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Players {
    entries: Vec<(u128, i32)>,
    next: i32,
}
impl Players {
    pub fn load(ids: &[i32], profiles: &[i64]) -> Result<Self, &'static str> {
        if ids.len().checked_mul(2) != Some(profiles.len()) {
            return Err("corrupt player ID array lengths");
        }
        let mut result = Self::default();
        let mut highest = -1;
        for (&id, uuid) in ids.iter().zip(profiles.as_chunks::<2>().0) {
            let uuid = ((uuid[0] as u64 as u128) << 64) | (uuid[1] as u64 as u128);
            highest = highest.max(id);
            // HashBiMap.put allows an existing key to move, but refuses a value
            // already held by a different key. Preserve source insertion order.
            if result.entries.iter().any(|&(u, i)| i == id && u != uuid) {
                return Err("duplicate player ID");
            }
            if let Some(entry) = result.entries.iter_mut().find(|(u, _)| *u == uuid) {
                entry.1 = id;
            } else {
                result.entries.push((uuid, id));
            }
        }
        result.next = highest.wrapping_add(1);
        Ok(result)
    }
    pub fn profile(&self, id: i32) -> Option<u128> {
        self.entries.iter().find(|(_, i)| *i == id).map(|(u, _)| *u)
    }
    pub fn id(&mut self, uuid: u128) -> Result<(i32, bool), &'static str> {
        if let Some((_, id)) = self.entries.iter().find(|(u, _)| *u == uuid) {
            return Ok((*id, false));
        }
        let id = self.next;
        self.next = self.next.wrapping_add(1);
        if self.profile(id).is_some() {
            return Err("duplicate allocated player ID");
        }
        self.entries.push((uuid, id));
        Ok((id, true))
    }
    pub fn save(&self) -> (Vec<i32>, Vec<i64>) {
        (
            self.entries.iter().map(|(_, id)| *id).collect(),
            self.entries
                .iter()
                .flat_map(|(u, _)| [(*u >> 64) as u64 as i64, *u as u64 as i64])
                .collect(),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_arrays_restore_identity_and_next_is_highest_loaded_plus_one() {
        let mut p = Players::default();
        assert_eq!(p.id(u128::MAX), Ok((0, true)));
        assert_eq!(p.id(0), Ok((1, true)));
        assert_eq!(p.id(u128::MAX), Ok((0, false)));
        let (ids, uuids) = p.save();
        let mut back = Players::load(&ids, &uuids).unwrap();
        assert_eq!(back, p);
        assert_eq!(back.profile(0), Some(u128::MAX));
        assert_eq!(back.id(5), Ok((2, true)));
        let mut p = Players::load(&[-9, 7], &[0, 1, 0, 2]).unwrap();
        assert_eq!(p.id(3), Ok((8, true)));
    }
    #[test]
    fn corrupt_mapping_is_refused_and_replaced_profile_retains_source_high_water_mark() {
        assert!(Players::load(&[1], &[0]).is_err());
        assert!(Players::load(&[1, 1], &[0, 1, 0, 2]).is_err());
        let mut p = Players::load(&[8, 3], &[0, 1, 0, 1]).unwrap();
        assert_eq!(p.profile(8), None);
        assert_eq!(p.id(1), Ok((3, false)));
        assert_eq!(p.id(2), Ok((9, true)));
    }
}
