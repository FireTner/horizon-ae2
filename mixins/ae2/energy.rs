// SPDX-License-Identifier: LGPL-3.0-or-later
// Port of appeng/me/energy/StoredEnergyAmount.java at upstream.lock.json.
// Copyright (c) Applied Energistics 2 contributors; 2026 Horizon port contributors.

pub const MIN_AMOUNT: f64 = 0.000001;
pub const MAX_MAXIMUM: f64 = (i64::MAX / 10000) as f64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerEvent {
    Provide,
    Receive,
}

#[derive(Clone, Debug)]
pub struct StoredEnergy {
    amount: f64,
    maximum: f64,
    provide_threshold: f64,
    receive_threshold: f64,
    providing: bool,
    receiving: bool,
}

impl StoredEnergy {
    pub fn new(amount: f64, maximum: f64) -> Self {
        assert!(maximum.is_finite() && (0.0..=MAX_MAXIMUM).contains(&maximum));
        assert!(amount.is_finite() && (0.0..=maximum).contains(&amount));
        Self {
            amount,
            maximum,
            provide_threshold: 0.1,
            receive_threshold: 0.1,
            // Upstream initially registers every node as both a provider and receiver.
            providing: true,
            receiving: true,
        }
    }

    pub fn amount(&self) -> f64 {
        self.amount
    }

    pub fn maximum(&self) -> f64 {
        self.maximum
    }

    pub fn remaining_capacity(&self) -> f64 {
        self.maximum - self.amount
    }

    /// Returns accepted energy. An IAEPowerStorage adapter returns amount - accepted.
    pub fn insert(&mut self, amount: f64, commit: bool) -> (f64, Vec<PowerEvent>) {
        if !amount.is_finite() || amount < MIN_AMOUNT {
            return (0.0, Vec::new());
        }
        let inserted = amount.min(self.remaining_capacity());
        let events = if commit {
            self.set_stored(self.amount + inserted)
        } else {
            Vec::new()
        };
        (inserted, events)
    }

    pub fn extract(&mut self, amount: f64, commit: bool) -> (f64, Vec<PowerEvent>) {
        if !amount.is_finite() || amount < MIN_AMOUNT {
            return (0.0, Vec::new());
        }
        let extracted = amount.min(self.amount);
        let events = if commit {
            self.set_stored(self.amount - extracted)
        } else {
            Vec::new()
        };
        (extracted, events)
    }

    pub fn set_stored(&mut self, mut amount: f64) -> Vec<PowerEvent> {
        assert!(amount.is_finite());
        if amount < MIN_AMOUNT {
            amount = 0.0;
        }
        self.amount = amount.clamp(0.0, self.maximum);
        self.events()
    }

    pub fn set_maximum(&mut self, maximum: f64) -> Vec<PowerEvent> {
        assert!(maximum.is_finite() && maximum >= 0.0);
        self.maximum = maximum.min(MAX_MAXIMUM);
        self.amount = self.amount.clamp(0.0, self.maximum);
        self.events()
    }

    fn events(&mut self) -> Vec<PowerEvent> {
        let provide = self.amount >= self.provide_threshold;
        let receive = self.remaining_capacity() >= self.receive_threshold;
        let mut events = Vec::new();
        if !self.providing && provide {
            events.push(PowerEvent::Provide);
        }
        if !self.receiving && receive {
            events.push(PowerEvent::Receive);
        }
        self.providing = provide;
        self.receiving = receive;
        events
    }
}

/// EnergyCellBlockEntity.getStorageLevelFromFillFactor (MAX_FULLNESS = 4).
pub fn display_level(fill_factor: f64) -> u8 {
    (4.0 * (fill_factor + 0.01).clamp(0.0, 1.0)).floor() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_never_changes_storage_or_events() {
        let mut e = StoredEnergy::new(5.0, 10.0);
        assert_eq!(e.insert(20.0, false), (5.0, vec![]));
        assert_eq!(e.extract(20.0, false), (5.0, vec![]));
        assert_eq!(e.amount(), 5.0);
    }

    #[test]
    fn capacity_and_precision_match_upstream() {
        let mut e = StoredEnergy::new(0.0, 10.0);
        assert_eq!(e.insert(-1.0, true).0, 0.0);
        assert_eq!(e.insert(MIN_AMOUNT / 2.0, true).0, 0.0);
        assert_eq!(e.insert(20.0, true).0, 10.0);
        assert_eq!(
            e.extract(10.0 - MIN_AMOUNT / 2.0, true).0,
            10.0 - MIN_AMOUNT / 2.0
        );
        assert_eq!(e.amount(), 0.0);
    }

    #[test]
    fn threshold_events_only_on_becoming_available() {
        let mut e = StoredEnergy::new(0.0, 10.0);
        assert!(e.set_stored(0.0).is_empty());
        assert_eq!(e.insert(0.1, true).1, vec![PowerEvent::Provide]);
        assert!(e.insert(9.9, true).1.is_empty());
        assert!(e.extract(0.05, true).1.is_empty());
        assert_eq!(e.extract(0.1, true).1, vec![PowerEvent::Receive]);
        assert!(e.extract(1.0, true).1.is_empty());
    }

    #[test]
    fn reducing_capacity_clamps_and_reenables_receiving_on_growth() {
        let mut e = StoredEnergy::new(10.0, 10.0);
        e.set_stored(10.0);
        e.set_maximum(3.0);
        assert_eq!(e.amount(), 3.0);
        assert_eq!(e.set_maximum(4.0), vec![PowerEvent::Receive]);
    }

    #[test]
    fn display_uses_one_percent_bias() {
        for (fill, level) in [
            (0.0, 0),
            (0.239, 0),
            (0.24, 1),
            (0.49, 2),
            (0.74, 3),
            (0.99, 4),
            (1.0, 4),
        ] {
            assert_eq!(display_level(fill), level);
        }
    }
}
