//! Attestation aggregate coverage gauges (leanSpec node registry):
//! validators covered per section and subnet, subnets covered per section,
//! and the block / timely delta. Recorded once per imported or proposed block.

use ethean_metrics::lean::set;
use ethean_types::AggregatedAttestation;

use crate::chain_owner::ChainOwner;

/// Union of participant bits over a set of attestations.
pub fn union_bits<'a>(attestations: impl Iterator<Item = &'a AggregatedAttestation>) -> Vec<bool> {
    let mut union = Vec::new();
    for a in attestations {
        for (i, bit) in a.aggregation_bits.bits.iter().enumerate() {
            if *bit {
                if union.len() <= i {
                    union.resize(i + 1, false);
                }
                union[i] = true;
            }
        }
    }
    union
}

/// Validators covered by the local aggregate pool (best proof per data).
pub fn pool_bits(owner: &ChainOwner) -> Vec<bool> {
    let decoded: Vec<AggregatedAttestation> = owner
        .aggregates
        .best_entries()
        .into_iter()
        .filter_map(|(_, e)| AggregatedAttestation::ssz_decode(&e.attestation_ssz).ok())
        .collect();
    union_bits(decoded.iter())
}

fn count(bits: &[bool]) -> u64 {
    bits.iter().filter(|b| **b).count() as u64
}

fn or(a: &[bool], b: &[bool]) -> Vec<bool> {
    let n = a.len().max(b.len());
    (0..n)
        .map(|i| a.get(i).copied().unwrap_or(false) || b.get(i).copied().unwrap_or(false))
        .collect()
}

fn only_in(a: &[bool], b: &[bool]) -> u64 {
    a.iter()
        .enumerate()
        .filter(|(i, set)| **set && !b.get(*i).copied().unwrap_or(false))
        .count() as u64
}

fn record_section(section: &str, bits: &[bool], committees: u64) {
    set(
        "lean_attestation_aggregate_coverage_validators",
        &[section, "combined"],
        count(bits) as f64,
    );
    let committees = committees.max(1);
    let mut subnets_covered = 0u64;
    for subnet in 0..committees {
        let n = bits
            .iter()
            .enumerate()
            .filter(|(i, set)| **set && (*i as u64) % committees == subnet)
            .count();
        if n > 0 {
            subnets_covered += 1;
        }
        let label = format!("subnet_{subnet}");
        set(
            "lean_attestation_aggregate_coverage_validators",
            &[section, label.as_str()],
            n as f64,
        );
    }
    set(
        "lean_attestation_aggregate_coverage_subnets",
        &[section],
        subnets_covered as f64,
    );
}

/// Record coverage for a block: `timely` is what the local pool held before
/// the block, `late` what arrived after it, `block` the block's payloads.
pub fn record_block_coverage(block: &[bool], timely: &[bool], late: &[bool], committees: u64) {
    record_section("block", block, committees);
    record_section("timely", timely, committees);
    record_section("late", late, committees);
    record_section("combined", &or(&or(block, timely), late), committees);
    set(
        "lean_attestation_aggregate_coverage_diff_validators",
        &["block_only"],
        only_in(block, timely) as f64,
    );
    set(
        "lean_attestation_aggregate_coverage_diff_validators",
        &["timely_only"],
        only_in(timely, block) as f64,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_metrics::lean::value;

    #[test]
    fn records_sections_subnets_and_deltas() {
        record_block_coverage(
            &[true, true, false, false],
            &[false, true, true, false],
            &[],
            2,
        );
        assert_eq!(
            value(
                "lean_attestation_aggregate_coverage_validators",
                &["block", "combined"]
            ),
            Some(2.0)
        );
        assert_eq!(
            value(
                "lean_attestation_aggregate_coverage_validators",
                &["block", "subnet_1"]
            ),
            Some(1.0)
        );
        assert_eq!(
            value("lean_attestation_aggregate_coverage_subnets", &["timely"]),
            Some(2.0)
        );
        assert_eq!(
            value(
                "lean_attestation_aggregate_coverage_validators",
                &["combined", "combined"]
            ),
            Some(3.0)
        );
        assert_eq!(
            value(
                "lean_attestation_aggregate_coverage_diff_validators",
                &["block_only"]
            ),
            Some(1.0)
        );
        assert_eq!(
            value(
                "lean_attestation_aggregate_coverage_diff_validators",
                &["timely_only"]
            ),
            Some(1.0)
        );
    }
}
