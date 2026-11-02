// SPDX-License-Identifier: LGPL-3.0-or-later
// PartPlacement.getPartPlacement/canPlacePartOnBlock for central cable parts.
// Segment hit selection affects side parts; cable admission is side-independent.
use super::ae2_grid::Direction;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CenterSlot {
    NoHost {
        replaceable: bool,
        may_interact: bool,
    },
    Host {
        occupied: bool,
        compatible: bool,
    },
}
impl CenterSlot {
    pub fn accepts(self) -> bool {
        match self {
            Self::NoHost {
                replaceable,
                may_interact,
            } => replaceable && may_interact,
            Self::Host {
                occupied,
                compatible,
            } => !occupied && compatible,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub pos: [i32; 3],
    pub side: Direction,
}
pub fn center_target(
    pos: [i32; 3],
    face: Direction,
    holding_ctrl: bool,
    mut at: impl FnMut([i32; 3]) -> CenterSlot,
) -> Option<Placement> {
    let side = if holding_ctrl { face.opposite() } else { face };
    if at(pos).accepts() {
        return Some(Placement { pos, side });
    }
    let delta = side.offset();
    let pos = std::array::from_fn(|i| pos[i].wrapping_add(delta[i]));
    at(pos).accepts().then_some(Placement {
        pos,
        side: side.opposite(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_prefers_clicked_position_then_opposite_side_of_adjacent_host() {
        let pos = [3, 4, 5];
        for side in Direction::ALL {
            for ctrl in [false, true] {
                let actual = if ctrl { side.opposite() } else { side };
                let mut visited = vec![];
                let placement = center_target(pos, side, ctrl, |p| {
                    visited.push(p);
                    CenterSlot::NoHost {
                        replaceable: true,
                        may_interact: true,
                    }
                })
                .unwrap();
                assert_eq!(placement, Placement { pos, side: actual });
                assert_eq!(visited, vec![pos]);
                let next = std::array::from_fn(|i| pos[i] + actual.offset()[i]);
                let placement = center_target(pos, side, ctrl, |p| CenterSlot::Host {
                    occupied: p == pos,
                    compatible: true,
                })
                .unwrap();
                assert_eq!(
                    placement,
                    Placement {
                        pos: next,
                        side: actual.opposite()
                    }
                );
                assert!(
                    center_target(pos, side, ctrl, |_| CenterSlot::Host {
                        occupied: true,
                        compatible: true
                    })
                    .is_none()
                );
            }
        }
    }
    #[test]
    fn existing_hosts_and_new_host_permission_follow_different_source_paths() {
        assert!(
            CenterSlot::Host {
                occupied: false,
                compatible: true
            }
            .accepts()
        );
        for replaceable in [false, true] {
            for may_interact in [false, true] {
                assert_eq!(
                    CenterSlot::NoHost {
                        replaceable,
                        may_interact
                    }
                    .accepts(),
                    replaceable && may_interact
                );
            }
        }
        assert!(
            !CenterSlot::Host {
                occupied: false,
                compatible: false
            }
            .accepts()
        );
    }
}
