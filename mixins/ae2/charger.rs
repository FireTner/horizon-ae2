// SPDX-License-Identifier: LGPL-3.0-or-later
// ChargerBlockEntity.doWork (recipe branch), CrankBlockEntity and
// AENetworkedPoweredBlockEntity.Crankable at upstream.lock.json.
// Copyright (c) AlgorithmX2 / AE2 contributors; 2026 Horizon port contributors.

pub const POWER_MAXIMUM: f64 = 1600.0;
pub const POWER_THRESHOLD: f64 = POWER_MAXIMUM - 1.0;
pub const POWER_PER_TURN: f64 = 160.0;
pub const TICKS_PER_ROTATION: i32 = 18;
pub const CHARGER_INTERVAL: u32 = 10; // TickRates.Charger(10, 10)

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Crank {
    pub rotation: i32,
    pub charge: i32,
    pub hits: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrankResult {
    Started,
    Busy,
    Rejected,
    Break,
}

impl Crank {
    /// `None` means no ICrankable. Eleven blocked uses break a connected crank.
    pub fn power(&mut self, can_turn: Option<bool>) -> CrankResult {
        if self.rotation >= 3 {
            return CrankResult::Busy;
        }
        match can_turn {
            Some(true) => {
                self.hits = 0;
                self.rotation += TICKS_PER_ROTATION;
                CrankResult::Started
            }
            Some(false) => {
                self.hits += 1;
                if self.hits > 10 {
                    CrankResult::Break
                } else {
                    CrankResult::Rejected
                }
            }
            None => CrankResult::Rejected,
        }
    }

    /// Returns true at the end of a turn, when ICrankable.applyTurn is called.
    pub fn tick(&mut self) -> bool {
        if self.rotation <= 0 {
            return false;
        }
        self.charge += 1;
        let turn = self.charge >= TICKS_PER_ROTATION;
        if turn {
            self.charge -= TICKS_PER_ROTATION;
        }
        self.rotation -= 1;
        turn
    }
}

pub fn can_turn(energy: f64) -> bool {
    // AENetworkedPoweredBlockEntity.Crankable.canTurn: see upstream source.
    energy < POWER_MAXIMUM
}

/// The recipe branch consumes the local maximum * powerMultiplier on success,
/// capped by stored energy. The adapter must supply Java-compatible randomFloat
/// samples for oracle comparisons; no invented fixed processing duration.
pub fn recipe_work(
    energy: &mut f64,
    has_recipe: bool,
    random: f32,
    power_multiplier: f64,
) -> (bool, bool) {
    let working = *energy >= POWER_THRESHOLD && has_recipe;
    let completed = working && random > 0.8;
    if completed {
        *energy = (*energy - POWER_MAXIMUM * power_multiplier).max(0.0);
    }
    (working, completed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_delivers_at_tick_eighteen_and_allows_two_tick_overlap() {
        let mut crank = Crank::default();
        assert_eq!(crank.power(Some(true)), CrankResult::Started);
        for _ in 0..15 {
            assert!(!crank.tick());
        }
        assert_eq!(crank.power(Some(true)), CrankResult::Busy);
        assert!(!crank.tick());
        assert_eq!(crank.power(Some(true)), CrankResult::Started);
        assert!(!crank.tick());
        assert!(crank.tick());
        assert_eq!(crank.rotation, 18);
    }

    #[test]
    fn rejected_turns_break_on_eleven_but_absent_target_does_not() {
        let mut c = Crank::default();
        for _ in 0..20 {
            assert_eq!(c.power(None), CrankResult::Rejected);
        }
        for _ in 0..10 {
            assert_eq!(c.power(Some(false)), CrankResult::Rejected);
        }
        assert_eq!(c.power(Some(false)), CrankResult::Break);
    }

    #[test]
    fn recipe_needs_1599_and_strictly_more_than_point_eight() {
        let mut power = 1598.0;
        assert_eq!(recipe_work(&mut power, true, 1.0, 1.0), (false, false));
        power = 1599.0;
        assert_eq!(recipe_work(&mut power, true, 0.8, 1.0), (true, false));
        assert_eq!(recipe_work(&mut power, false, 0.9, 1.0), (false, false));
        assert_eq!(recipe_work(&mut power, true, 0.80001, 1.0), (true, true));
        assert_eq!(power, 0.0);
    }

    #[test]
    fn configured_multiplier_applies_to_recipe_cost() {
        let mut power = 1600.0;
        recipe_work(&mut power, true, 0.9, 0.5);
        assert_eq!(power, 800.0);
    }
}
