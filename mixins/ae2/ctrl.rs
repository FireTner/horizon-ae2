// SPDX-License-Identifier: LGPL-3.0-or-later
// UpdateHoldingCtrlPacket and the non-persistent HOLDING_CTRL player attachment.
use std::collections::BTreeSet;
pub const CHANNEL: &str = "ae2:toggle_ctrl_down";
pub const BINDING: &str = "part_placement_opposite";
pub const DEFAULT_KEY: &str = "key.keyboard.left.control";
pub fn decode(data: &[u8]) -> Option<bool> {
    // FriendlyByteBuf.readBoolean consumes one byte; every nonzero byte is true.
    data.first().map(|value| *value != 0)
}
pub fn encode(down: bool) -> [u8; 1] {
    [u8::from(down)]
}
#[derive(Clone, Debug, Default)]
pub struct Controls {
    down: BTreeSet<u128>,
}
impl Controls {
    pub fn receive(&mut self, sender: u128, data: &[u8]) -> bool {
        let Some(down) = decode(data) else {
            return false;
        };
        if down {
            self.down.insert(sender);
        } else {
            self.down.remove(&sender);
        }
        true
    }
    pub fn down(&self, player: u128) -> bool {
        self.down.contains(&player)
    }
    pub fn reset(&mut self, player: u128) {
        self.down.remove(&player);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_boolean_codec_handles_all_byte_values_and_empty_does_not_mutate() {
        for value in 0..=255u8 {
            assert_eq!(decode(&[value]), Some(value != 0));
        }
        assert_eq!(decode(&[]), None);
        for down in [false, true] {
            assert_eq!(decode(&encode(down)), Some(down));
        }
        assert_eq!(decode(&[0, 255]), Some(false));
        assert_eq!(decode(&[255, 0]), Some(true));
    }
    #[test]
    fn attachments_are_per_sender_and_reset_on_a_new_player_entity() {
        let mut controls = Controls::default();
        assert!(!controls.down(1));
        assert!(!controls.down(2));
        controls.receive(1, &[1]);
        assert!(controls.down(1));
        assert!(!controls.down(2));
        assert!(!controls.receive(1, &[]));
        assert!(controls.down(1));
        controls.receive(2, &[9]);
        controls.receive(1, &[0]);
        assert!(!controls.down(1));
        assert!(controls.down(2));
        controls.reset(2);
        assert!(!controls.down(2));
    }
}
