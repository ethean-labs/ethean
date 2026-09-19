//! Simple gossip score deltas for application feedback.

/// Delta applied on ACCEPT (slightly positive).
pub const SCORE_ACCEPT: i32 = 1;

/// Delta applied on IGNORE (neutral).
pub const SCORE_IGNORE: i32 = 0;

/// Delta applied on REJECT (penalty).
pub const SCORE_REJECT: i32 = -25;

/// Map validation action to score delta.
pub fn delta_for(action: crate::gossip::validation::GossipAction) -> i32 {
    use crate::gossip::validation::GossipAction::*;
    match action {
        Accept => SCORE_ACCEPT,
        Ignore => SCORE_IGNORE,
        Reject => SCORE_REJECT,
    }
}
