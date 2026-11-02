// SPDX-License-Identifier: LGPL-3.0-or-later
// PartPlacement.place: ordered result, failure update and server-only consumption.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UseResult {
    Pass,
    Fail,
    Success,
    Consume,
}
impl UseResult {
    pub fn stops_click(self) -> bool {
        self != Self::Pass
    }
    pub fn swings(self) -> bool {
        self == Self::Success
    }
}
pub trait Context {
    type Placement;
    fn is_part(&self) -> bool;
    fn client_side(&self) -> bool;
    fn placement(&mut self) -> Option<Self::Placement>;
    fn place(&mut self, target: Self::Placement) -> bool;
    fn immediate_update(&mut self);
    // Implement the nullable-player/creative check without touching client items.
    fn consume(&mut self);
}
pub fn use_item(context: &mut impl Context) -> UseResult {
    if !context.is_part() {
        return UseResult::Pass;
    }
    let Some(target) = context.placement() else {
        return UseResult::Fail;
    };
    if !context.place(target) {
        context.immediate_update();
        return UseResult::Fail;
    }
    if context.client_side() {
        UseResult::Success
    } else {
        context.consume();
        UseResult::Consume
    }
}
