// SPDX-License-Identifier: LGPL-3.0-or-later
// CableBusStorage.java: center + lazily allocated six side/facade references.
use super::ae2_grid::Direction;
#[derive(Clone, Debug)]
pub struct PartStorage<C, P, F> {
    center: Option<C>,
    parts: Option<Box<[Option<P>; 6]>>,
    facades: Option<Box<[Option<F>; 6]>>,
}
impl<C, P, F> Default for PartStorage<C, P, F> {
    fn default() -> Self {
        Self {
            center: None,
            parts: None,
            facades: None,
        }
    }
}
impl<C, P, F> PartStorage<C, P, F> {
    pub fn center(&self) -> Option<&C> {
        self.center.as_ref()
    }
    pub fn set_center(&mut self, center: Option<C>) {
        self.center = center;
    }
    pub fn part(&self, side: Direction) -> Option<&P> {
        self.parts.as_ref()?.get(side as usize)?.as_ref()
    }
    /// Setting null still allocates the parts array in the original class.
    pub fn set_part(&mut self, side: Direction, part: Option<P>) {
        self.parts
            .get_or_insert_with(|| Box::new(std::array::from_fn(|_| None)))[side as usize] = part;
    }
    pub fn remove_part(&mut self, side: Direction) {
        if let Some(parts) = &mut self.parts {
            parts[side as usize] = None;
            if parts.iter().all(Option::is_none) {
                self.parts = None;
            }
        }
    }
    pub fn facade(&self, side: Direction) -> Option<&F> {
        self.facades.as_ref()?.get(side as usize)?.as_ref()
    }
    pub fn set_facade(&mut self, side: Direction, facade: Option<F>) {
        if facade.is_none() {
            self.remove_facade(side);
        } else {
            self.facades
                .get_or_insert_with(|| Box::new(std::array::from_fn(|_| None)))[side as usize] =
                facade;
        }
    }
    pub fn remove_facade(&mut self, side: Direction) {
        if let Some(facades) = &mut self.facades {
            facades[side as usize] = None;
            if facades.iter().all(Option::is_none) {
                self.facades = None;
            }
        }
    }
    pub fn parts_allocated(&self) -> bool {
        self.parts.is_some()
    }
    pub fn facades_allocated(&self) -> bool {
        self.facades.is_some()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn null_parts_and_null_facades_have_distinct_source_allocation_rules() {
        let mut storage: PartStorage<i32, i32, i32> = PartStorage::default();
        storage.set_part(Direction::Down, None);
        assert!(storage.parts_allocated());
        storage.set_facade(Direction::Down, None);
        assert!(!storage.facades_allocated());
        storage.set_center(Some(10));
        storage.set_part(Direction::East, Some(20));
        storage.set_facade(Direction::West, Some(30));
        storage.remove_part(Direction::Down);
        assert!(storage.parts_allocated());
        storage.remove_part(Direction::East);
        assert!(!storage.parts_allocated());
        storage.remove_facade(Direction::West);
        assert!(!storage.facades_allocated());
        assert_eq!(storage.center(), Some(&10));
    }
}
