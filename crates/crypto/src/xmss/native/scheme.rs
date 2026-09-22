//! Generalized XMSS: key generation, signing, verification, window advance.
//!
//! Follows leanSpec `xmss/interface.py` step for step. Signing is
//! deterministic in `(secret key, epoch, message)`.

use super::encoding::target_sum_encode;
use super::keys::{XmssPublicKey, XmssSecretKey, XmssSignature};
use super::leaves::bottom_tree_from_prf;
use super::merkle::{combined_path, verify_path, HashSubTree};
use super::params::{SchemeParams, MAX_TRIES, MESSAGE_LEN};
use super::prf::{chain_start, randomness};
use super::rand::{RandomExt, RandomSource};
use super::tweak_hash::{chain, hash_leaf, leaf_capacity, Digest};
use crate::error::{CryptoError, Result};

/// Align the requested window to whole bottom trees (at least two), clamped
/// to the lifetime. Returns `(start_bottom_index, end_bottom_index)`.
pub(crate) fn expand_activation_time(
    params: &SchemeParams,
    activation_epoch: u64,
    num_active_epochs: u64,
) -> (u64, u64) {
    let lifetime = params.lifetime();
    let c = params.leaves_per_bottom_tree();
    let mask = !(c - 1);
    let mut start = activation_epoch & mask;
    let mut end = (activation_epoch + num_active_epochs + c - 1) & mask;
    if end - start < 2 * c {
        end = start + 2 * c;
    }
    if end > lifetime {
        let duration = end - start;
        if duration > lifetime {
            start = 0;
            end = lifetime;
        } else {
            end = lifetime;
            start = (lifetime - duration) & mask;
        }
    }
    (start / c, end / c)
}

/// Generate a key pair active for `[activation_epoch, activation_epoch + num_active_epochs)`.
pub fn key_gen(
    params: &SchemeParams,
    rng: &mut dyn RandomSource,
    activation_epoch: u64,
    num_active_epochs: u64,
) -> Result<(XmssPublicKey, XmssSecretKey)> {
    if num_active_epochs == 0 {
        return Err(CryptoError::KeyGenerationFailed(
            "num_active_epochs must be non-zero".into(),
        ));
    }
    let end = activation_epoch
        .checked_add(num_active_epochs)
        .ok_or_else(|| CryptoError::KeyGenerationFailed("activation overflow".into()))?;
    if end > params.lifetime() {
        return Err(CryptoError::KeyGenerationFailed(format!(
            "requested interval [{activation_epoch}, {end}) exceeds lifetime {}",
            params.lifetime()
        )));
    }
    let (start_index, end_index) =
        expand_activation_time(params, activation_epoch, num_active_epochs);
    let w = params.leaves_per_bottom_tree();
    let parameter = rng.parameter()?;
    let prf_key = rng.prf_key()?;

    let left = bottom_tree_from_prf(params, &prf_key, &parameter, start_index)?;
    let right = bottom_tree_from_prf(params, &prf_key, &parameter, start_index + 1)?;
    let mut roots = vec![left.root(), right.root()];
    for index in start_index + 2..end_index {
        roots.push(bottom_tree_from_prf(params, &prf_key, &parameter, index)?.root());
    }
    let top_tree = HashSubTree::new_top_tree(rng, params, start_index, &parameter, roots)?;

    let sk = XmssSecretKey {
        prf_key,
        parameter,
        activation_epoch: start_index * w,
        num_active_epochs: (end_index - start_index) * w,
        top_tree,
        left_bottom_tree_index: start_index,
        left_bottom_tree: left,
        right_bottom_tree: right,
    };
    let pk = sk.public_key();
    Ok((pk, sk))
}

/// Slide the prepared window one bottom tree to the right, if possible.
/// Returns `false` when the window already reaches the activation end.
pub fn advance_preparation(params: &SchemeParams, sk: &mut XmssSecretKey) -> Result<bool> {
    let w = params.leaves_per_bottom_tree();
    let next_end = (sk.left_bottom_tree_index + 3) * w;
    if next_end > sk.activation_interval().end {
        return Ok(false);
    }
    let fresh = bottom_tree_from_prf(
        params,
        &sk.prf_key,
        &sk.parameter,
        sk.left_bottom_tree_index + 2,
    )?;
    sk.left_bottom_tree = std::mem::replace(&mut sk.right_bottom_tree, fresh);
    sk.left_bottom_tree_index += 1;
    Ok(true)
}

/// Advance the window until `epoch` is prepared (no-op if already prepared).
pub fn prepare_for_epoch(params: &SchemeParams, sk: &mut XmssSecretKey, epoch: u64) -> Result<()> {
    let activation = sk.activation_interval();
    if !activation.contains(&epoch) {
        return Err(CryptoError::EpochOutsideActivation {
            epoch,
            start: activation.start,
            end: activation.end,
        });
    }
    while !sk.prepared_interval(params).contains(&epoch) {
        if !advance_preparation(params, sk)? {
            return Err(CryptoError::SigningFailed("cannot prepare epoch".into()));
        }
    }
    Ok(())
}

/// Sign `message` at `epoch`; the key must already be prepared for `epoch`.
pub fn sign(
    params: &SchemeParams,
    sk: &XmssSecretKey,
    epoch: u32,
    message: &[u8; MESSAGE_LEN],
) -> Result<XmssSignature> {
    let epoch64 = epoch as u64;
    let activation = sk.activation_interval();
    if !activation.contains(&epoch64) {
        return Err(CryptoError::EpochOutsideActivation {
            epoch: epoch64,
            start: activation.start,
            end: activation.end,
        });
    }
    let prepared = sk.prepared_interval(params);
    if !prepared.contains(&epoch64) {
        return Err(CryptoError::SigningFailed(format!(
            "epoch {epoch} not prepared; window is [{}, {})",
            prepared.start, prepared.end
        )));
    }

    let mut found = None;
    for counter in 0..MAX_TRIES {
        let rho = randomness(&sk.prf_key, epoch, message, counter);
        if let Some(digits) = target_sum_encode(params, &sk.parameter, epoch, &rho, message) {
            found = Some((rho, digits));
            break;
        }
    }
    let (rho, digits) = found.ok_or_else(|| {
        CryptoError::SigningFailed(format!("no codeword after {MAX_TRIES} attempts"))
    })?;

    let hashes: Vec<Digest> = digits
        .iter()
        .enumerate()
        .map(|(i, &steps)| {
            let start = chain_start(&sk.prf_key, epoch, i as u64);
            chain(&sk.parameter, epoch, i as u8, 0, steps as usize, &start)
        })
        .collect();

    let boundary = prepared.start + params.leaves_per_bottom_tree();
    let bottom = if epoch64 < boundary {
        &sk.left_bottom_tree
    } else {
        &sk.right_bottom_tree
    };
    let path = combined_path(&sk.top_tree, bottom, epoch64);
    Ok(XmssSignature { path, rho, hashes })
}

/// Verify a decoded signature. Never panics on attacker-controlled input.
pub fn verify(
    params: &SchemeParams,
    pk: &XmssPublicKey,
    epoch: u32,
    message: &[u8; MESSAGE_LEN],
    sig: &XmssSignature,
) -> bool {
    if (epoch as u64) >= params.lifetime()
        || sig.hashes.len() != params.dimension
        || sig.path.siblings.len() != params.log_lifetime as usize
    {
        return false;
    }
    let Some(digits) = target_sum_encode(params, &pk.parameter, epoch, &sig.rho, message) else {
        return false;
    };
    let last = params.chain_length() - 1;
    let ends: Vec<Digest> = digits
        .iter()
        .zip(sig.hashes.iter())
        .enumerate()
        .map(|(i, (&digit, start))| {
            chain(
                &pk.parameter,
                epoch,
                i as u8,
                digit,
                last - digit as usize,
                start,
            )
        })
        .collect();
    let leaf = hash_leaf(&pk.parameter, &leaf_capacity(params), epoch, &ends);
    verify_path(&pk.parameter, &pk.root, epoch as u64, leaf, &sig.path)
}
