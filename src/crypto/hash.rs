//! Hash functions for Beam Chain
//!
//! Provides Poseidon hash for ZK-friendly operations and fallback SHA-256.

use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

/// Hash output type - 32 bytes
pub type Hash = [u8; 32];

/// Poseidon hash parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoseidonParams {
    /// Field prime for arithmetic
    pub field_prime: u64,
    /// Number of full rounds
    pub full_rounds: u8,
    /// Number of partial rounds
    pub partial_rounds: u8,
    /// S-box degree
    pub sbox_degree: u8,
}

impl Default for PoseidonParams {
    fn default() -> Self {
        Self {
            field_prime: 0x73eda753299d7d48, // Simplified field prime
            full_rounds: 8,
            partial_rounds: 56,
            sbox_degree: 5,
        }
    }
}

/// Poseidon hash function (ZK-friendly)
/// 
/// This is a simplified implementation for development.
/// Production should use a full cryptographic library.
pub fn poseidon_hash(input: &[u8]) -> Vec<u8> {
    // Simple but effective hash based on SHA-256 with domain separation
    let mut data = Vec::with_capacity(input.len() + 8);
    data.extend_from_slice(b"POSEIDON");  // Domain separator
    data.extend_from_slice(input);
    
    let hash = sha256(&data);
    
    // Apply simple transformation for differentiation
    let mut result = hash.to_vec();
    for i in 0..result.len() {
        result[i] = result[i].wrapping_add((i as u8).wrapping_mul(7));
    }
    
    result
}

/// Simplified Poseidon permutation
#[allow(dead_code)]
fn poseidon_permutation(state: &mut [u64], params: &PoseidonParams) {
    // This is a placeholder implementation
    // Real Poseidon requires proper round constants and MDS matrix
    
    for round in 0..params.full_rounds {
        // Add round constants with more variation
        for i in 0..state.len() {
            state[i] = state[i].wrapping_add((round as u64 * 0x123456789abcdef0) + (i as u64 * 0xfedcba9876543210));
        }
        
        // S-box layer
        for element in state.iter_mut() {
            *element = sbox(*element, params.sbox_degree);
        }
        
        // Linear layer (simplified MDS multiplication with better mixing)
        let temp = state.to_vec();
        for i in 0..state.len() {
            state[i] = temp.iter().enumerate()
                .map(|(j, &val)| {
                    // Use safer mixing coefficients to avoid overflow
                    let coeff = ((i * 7 + j * 11 + 1) % 251) as u64; // Prime modulus
                    val.wrapping_mul(coeff).wrapping_add(val >> 13)
                })
                .fold(0u64, |acc, x| acc.wrapping_add(x));
        }
    }
}

/// S-box function for Poseidon
#[allow(dead_code)]
fn sbox(input: u64, degree: u8) -> u64 {
    match degree {
        3 => input.wrapping_mul(input).wrapping_mul(input),
        5 => {
            let square = input.wrapping_mul(input);
            square.wrapping_mul(square).wrapping_mul(input)
        },
        _ => input.wrapping_mul(input).wrapping_mul(input), // Default to cube
    }
}

/// SHA-256 hash function (fallback/compatibility)
pub fn sha256(input: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

/// Merkle tree hashing with Poseidon
pub fn merkle_hash(left: &[u8], right: &[u8]) -> Vec<u8> {
    let mut combined = Vec::with_capacity(left.len() + right.len());
    combined.extend_from_slice(left);
    combined.extend_from_slice(right);
    poseidon_hash(&combined)
}

/// Hash chain function for WOTS
pub fn hash_chain(input: &[u8], iterations: u32) -> Vec<u8> {
    let mut current = input.to_vec();
    for _ in 0..iterations {
        current = poseidon_hash(&current);
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poseidon_hash_deterministic() {
        let input = b"Hello Beam Chain!";
        let hash1 = poseidon_hash(input);
        let hash2 = poseidon_hash(input);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_poseidon_hash_different_inputs() {
        let input1 = b"Hello Beam Chain!";
        let input2 = b"Hello Beam Chain?";
        let hash1 = poseidon_hash(input1);
        let hash2 = poseidon_hash(input2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_sha256_hash() {
        let input = b"test";
        let hash = sha256(input);
        assert_eq!(hash.len(), 32);
        
        // SHA-256 should be deterministic
        let hash2 = sha256(input);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_merkle_hash() {
        let left = b"left node";
        let right = b"right node";
        let merkle = merkle_hash(left, right);
        
        assert_eq!(merkle.len(), 32);
        
        // Should be deterministic
        let merkle2 = merkle_hash(left, right);
        assert_eq!(merkle, merkle2);
        
        // Order should matter
        let merkle_reversed = merkle_hash(right, left);
        assert_ne!(merkle, merkle_reversed);
    }

    #[test]
    fn test_hash_chain() {
        let input = b"test input";
        let chain0 = hash_chain(input, 0);
        let chain1 = hash_chain(input, 1);
        let chain2 = hash_chain(input, 2);
        
        assert_eq!(chain0, input);
        assert_ne!(chain1, chain0);
        assert_ne!(chain2, chain1);
        
        // chain2 should equal hash(chain1)
        let expected_chain2 = poseidon_hash(&chain1);
        assert_eq!(chain2, expected_chain2);
    }

    #[test]
    fn test_poseidon_params() {
        let params = PoseidonParams::default();
        assert_eq!(params.full_rounds, 8);
        assert_eq!(params.partial_rounds, 56);
        assert_eq!(params.sbox_degree, 5);
    }
}
