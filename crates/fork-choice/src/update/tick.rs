//! Interval ticking (leanSpec `timeline.py`).

use crate::error::ForkChoiceError;
use crate::store::ForkChoiceStore;

impl ForkChoiceStore {
    /// Advance store time to `target` intervals, running each interval's actions.
    ///
    /// `has_proposal` gates interval-0 vote promotion on the final tick only
    /// (leanSpec `on_tick` semantics).
    pub fn on_tick(&mut self, target: u64) -> Result<(), ForkChoiceError> {
        self.on_tick_with(target, true)
    }

    /// Same as [`Self::on_tick`] with an explicit proposal flag.
    pub fn on_tick_with(
        &mut self,
        target: u64,
        has_proposal: bool,
    ) -> Result<(), ForkChoiceError> {
        if target < self.time {
            return Err(ForkChoiceError::TickInPast);
        }
        while self.time < target {
            let next = self.time + 1;
            let signal = has_proposal && next == target;
            self.tick_interval(signal)?;
        }
        Ok(())
    }

    /// Advance one interval and apply interval-specific store actions.
    pub fn tick_interval(&mut self, has_proposal: bool) -> Result<(), ForkChoiceError> {
        self.time += 1;
        let within = self.time % self.intervals_per_slot;
        match within {
            0 if has_proposal => self.accept_new_attestations()?,
            3 => self.update_safe_target()?,
            4 => self.accept_new_attestations()?,
            _ => {}
        }
        Ok(())
    }
}
