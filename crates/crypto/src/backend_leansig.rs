//! leanSig PROD backend (feature `leansig-backend`).

use leansig::serialization::Serializable;
use leansig::signature::generalized_xmss::instantiations_aborting::lifetime_2_to_the_32::{
    SIGAbortingTargetSumLifetime32Dim46Base8 as ProdScheme,
};
use leansig::signature::SignatureScheme;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::error::{CryptoError, Result};
use crate::signature::{PublicKey, Signature};
use crate::xmss::config::{MESSAGE_BYTES, PUBLIC_KEY_BYTES, SIGNATURE_BYTES};
use super::SecretKeyMaterial;

/// Cap active epochs for in-process key_gen (full 2^32 is operator-scale).
pub const MAX_WRAPPER_ACTIVE_EPOCHS: u32 = 2;

pub fn key_gen(
    activation_epoch: u32,
    num_active_epochs: u32,
) -> Result<(PublicKey, SecretKeyMaterial)> {
    if num_active_epochs == 0 || num_active_epochs > MAX_WRAPPER_ACTIVE_EPOCHS {
        return Err(CryptoError::KeyGenerationFailed(format!(
            "leansig PROD key_gen allows 1..={MAX_WRAPPER_ACTIVE_EPOCHS} active epochs \
             (requested {num_active_epochs})"
        )));
    }
    let mut rng = StdRng::from_os_rng();
    let (pk, sk) = ProdScheme::key_gen(
        &mut rng,
        activation_epoch as usize,
        num_active_epochs as usize,
    );
    let pk_bytes = pk.to_bytes();
    if pk_bytes.len() != PUBLIC_KEY_BYTES {
        return Err(CryptoError::InvalidPublicKeyLength {
            expected: PUBLIC_KEY_BYTES,
            got: pk_bytes.len(),
        });
    }
    let mut arr = [0u8; PUBLIC_KEY_BYTES];
    arr.copy_from_slice(&pk_bytes);
    Ok((
        PublicKey::from_bytes(arr),
        SecretKeyMaterial {
            bytes: sk.to_bytes(),
            activation_epoch,
            num_active_epochs,
        },
    ))
}

pub fn sign(
    sk: &SecretKeyMaterial,
    epoch: u32,
    message: &[u8; MESSAGE_BYTES],
) -> Result<Signature> {
    type Sk = <ProdScheme as SignatureScheme>::SecretKey;
    let secret = Sk::from_bytes(&sk.bytes)
        .map_err(|e| CryptoError::SigningFailed(format!("decode secret key: {e:?}")))?;
    let sig = ProdScheme::sign(&secret, epoch, message)
        .map_err(|e| CryptoError::SigningFailed(e.to_string()))?;
    wire_signature_from_leansig(&sig.to_bytes())
}

pub fn verify(
    pk: &PublicKey,
    epoch: u32,
    message: &[u8; MESSAGE_BYTES],
    signature: &Signature,
) -> Result<bool> {
    type Pk = <ProdScheme as SignatureScheme>::PublicKey;
    type Sig = <ProdScheme as SignatureScheme>::Signature;
    let public = Pk::from_bytes(pk.as_bytes())
        .map_err(|e| CryptoError::SigningFailed(format!("decode public key: {e:?}")))?;
    let raw = leansig_bytes_from_wire(signature.as_bytes())?;
    let sig = Sig::from_bytes(&raw)
        .map_err(|e| CryptoError::SigningFailed(format!("decode signature: {e:?}")))?;
    Ok(ProdScheme::verify(&public, epoch, message, &sig))
}

fn wire_signature_from_leansig(raw: &[u8]) -> Result<Signature> {
    if raw.len() == SIGNATURE_BYTES {
        return Signature::try_from_slice(raw);
    }
    if raw.len() + 4 <= SIGNATURE_BYTES {
        let mut framed = [0u8; SIGNATURE_BYTES];
        framed[..4].copy_from_slice(&(raw.len() as u32).to_le_bytes());
        framed[4..4 + raw.len()].copy_from_slice(raw);
        return Ok(Signature::from_bytes(framed));
    }
    Err(CryptoError::InvalidSignatureLength {
        expected: SIGNATURE_BYTES,
        got: raw.len(),
    })
}

fn leansig_bytes_from_wire(wire: &[u8; SIGNATURE_BYTES]) -> Result<Vec<u8>> {
    let len = u32::from_le_bytes(wire[..4].try_into().unwrap()) as usize;
    if len > 0 && len + 4 <= SIGNATURE_BYTES && wire[4 + len..].iter().all(|&b| b == 0) {
        return Ok(wire[4..4 + len].to_vec());
    }
    Ok(wire.to_vec())
}
