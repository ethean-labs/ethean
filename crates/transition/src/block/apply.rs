//! Apply block body after header validation.

use ethean_types::{Block, State};

use crate::context::TransitionContext;
use crate::error::TransitionError;
use crate::operation::attestation::process_attestations;

use super::validate::process_block_header;

/// Apply full block processing: header then attestations (leanSpec `process_block`).
pub fn process_block(
    state: &mut State,
    block: &Block,
    ctx: &TransitionContext,
) -> Result<(), TransitionError> {
    process_block_header(state, block)?;
    process_attestations(state, &block.body.attestations, ctx)?;
    Ok(())
}
