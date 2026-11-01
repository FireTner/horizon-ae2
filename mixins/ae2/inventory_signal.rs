// SPDX-License-Identifier: LGPL-3.0-or-later
// InternalInventory -> ContainerAdapter -> AbstractContainerMenu. Both native
// hosts have slot limit one; min(container limit, valid item limit) is thus one.
pub fn signal(counts: impl IntoIterator<Item = u32>, slots: usize) -> i32 {
    if slots == 0 {
        return 0;
    }
    let mut fullness = 0.0_f32;
    for count in counts.into_iter().take(slots) {
        fullness += count as f32;
    }
    fullness /= slots as f32;
    // Mth.lerpDiscrete(fullness, 0, 15), without an invented fullness clamp.
    ((fullness * 14.0).floor() as i32).wrapping_add(i32::from(fullness > 0.0))
}

#[cfg(test)]
mod tests {
    use super::signal;

    #[test]
    fn all_drive_occupancies_use_ten_real_slots_and_ignore_transport_padding() {
        let expected = [0, 2, 3, 5, 6, 8, 9, 10, 12, 13, 15];
        for mask in 0_u32..1024 {
            let counts = (0..18).map(|slot| if slot < 10 { (mask >> slot) & 1 } else { 99 });
            assert_eq!(signal(counts, 10), expected[mask.count_ones() as usize]);
        }
    }

    #[test]
    fn direct_source_overstacks_are_not_clamped_to_the_normal_insertion_limit() {
        for count in 0..=99 {
            assert_eq!(
                signal([count], 1),
                (count * 14 + u32::from(count > 0)) as i32
            );
        }
        for total in 0..=990 {
            assert_eq!(
                signal([total], 10),
                (total * 14 / 10 + u32::from(total > 0)) as i32
            );
        }
    }

    #[test]
    fn missing_or_zero_count_slots_are_empty() {
        assert_eq!(signal([], 1), 0);
        assert_eq!(signal([0; 18], 10), 0);
        assert_eq!(signal([99], 0), 0);
    }
}
