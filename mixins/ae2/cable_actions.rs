// SPDX-License-Identifier: LGPL-3.0-or-later
// Scoped result bridge for Horizon's boolean hand APIs. Never persists on a player.
use super::{Game, ae2_part_use::UseResult};
use horizon_world::nbt::Tag;
use std::cell::RefCell;
struct Frame {
    game: usize,
    result: Option<UseResult>,
}
thread_local! { static FRAMES: RefCell<Vec<Frame>> = const { RefCell::new(Vec::new()) }; }
struct Packet {
    game: usize,
    updates: Vec<([i32; 3], Tag)>,
}
thread_local! { static PACKETS: RefCell<Vec<Packet>> = const { RefCell::new(Vec::new()) }; }
pub(super) struct PacketScope;
impl Drop for PacketScope {
    fn drop(&mut self) {
        PACKETS.with(|p| {
            p.borrow_mut().pop();
        });
    }
}
pub(super) fn packet_begin(game: &Game<'_>) -> Option<PacketScope> {
    PACKETS.with(|p| {
        let mut p = p.borrow_mut();
        if p.iter().any(|p| p.game == address(game)) {
            return None;
        }
        p.push(Packet {
            game: address(game),
            updates: vec![],
        });
        Some(PacketScope)
    })
}
pub(super) fn queue_update(game: &Game<'_>, pos: [i32; 3], tag: Tag) {
    PACKETS.with(|p| {
        if let Some(p) = p
            .borrow_mut()
            .last_mut()
            .filter(|p| p.game == address(game))
        {
            p.updates.push((pos, tag));
        }
    });
}
pub(super) fn take_updates(game: &Game<'_>) -> Vec<([i32; 3], Tag)> {
    PACKETS.with(|p| {
        p.borrow_mut()
            .last_mut()
            .filter(|p| p.game == address(game))
            .map_or_else(Vec::new, |p| std::mem::take(&mut p.updates))
    })
}
pub(super) struct Scope;
impl Drop for Scope {
    fn drop(&mut self) {
        FRAMES.with(|frames| {
            frames.borrow_mut().pop();
        });
    }
}
fn address(game: &Game<'_>) -> usize {
    std::ptr::from_ref(game).addr()
}
fn enter(game: usize) -> Option<Scope> {
    FRAMES.with(|frames| {
        let mut frames = frames.borrow_mut();
        if frames.iter().any(|frame| frame.game == game) {
            return None;
        }
        frames.push(Frame { game, result: None });
        Some(Scope)
    })
}
pub(super) fn begin(game: &Game<'_>) -> Option<Scope> {
    enter(address(game))
}
pub(super) fn record(game: &Game<'_>, result: UseResult) {
    FRAMES.with(|frames| {
        if let Some(frame) = frames
            .borrow_mut()
            .last_mut()
            .filter(|f| f.game == address(game))
        {
            frame.result = Some(result);
        }
    });
}
pub(super) fn result(game: &Game<'_>) -> Option<UseResult> {
    FRAMES.with(|frames| {
        frames
            .borrow()
            .last()
            .filter(|f| f.game == address(game))
            .and_then(|f| f.result)
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recursion_is_bypassed_and_nested_games_are_isolated_with_scope_cleanup() {
        let first = enter(1).unwrap();
        assert!(enter(1).is_none());
        FRAMES
            .with(|frames| frames.borrow_mut().last_mut().unwrap().result = Some(UseResult::Fail));
        let second = enter(2).unwrap();
        FRAMES.with(|frames| assert_eq!(frames.borrow().last().unwrap().result, None));
        drop(second);
        FRAMES.with(|frames| {
            assert_eq!(
                frames.borrow().last().unwrap().result,
                Some(UseResult::Fail)
            )
        });
        drop(first);
        assert!(FRAMES.with(|frames| frames.borrow().is_empty()));
    }
}
