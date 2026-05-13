//! Deterministic fingerprint helpers for alerts.
//!
//! A fingerprint identifies the *meaning* of an alert independently of
//! when it was first seen. Two runs of the same rule against the same
//! entity must produce identical fingerprints so the engine can update
//! `last_seen_at` rather than insert a duplicate row.
//!
//! Fingerprints are deliberately human-readable so that they can be
//! eyeballed during debugging and tests can use direct comparison.

use std::fmt::Write as _;

/// Builds a colon-delimited fingerprint of the form
/// `<rule_name>:<source_entity_type>:<source_entity_id>:<extra...>`.
///
/// All segments are passed through [`sanitize_segment`] so that future
/// callers cannot accidentally break fingerprint parsing by including a
/// colon in an identifier.
pub fn build(rule_name: &str, segments: &[&str]) -> String {
    let mut out = sanitize_segment(rule_name);
    for segment in segments {
        out.push(':');
        let _ = write!(out, "{}", sanitize_segment(segment));
    }
    out
}

/// Replaces colons in a fingerprint segment with `-`. The colon is the
/// segment delimiter, so any identifier containing one would otherwise
/// collide with a multi-segment fingerprint.
pub fn sanitize_segment(segment: &str) -> String {
    segment.replace(':', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_assembles_segments_in_order() {
        let fp = build("StaleManualValuation", &["asset", "abc-123"]);
        assert_eq!(fp, "StaleManualValuation:asset:abc-123");
    }

    #[test]
    fn build_with_no_segments_returns_rule_name() {
        let fp = build("OrphanedRule", &[]);
        assert_eq!(fp, "OrphanedRule");
    }

    #[test]
    fn build_sanitizes_colons_so_segments_remain_unambiguous() {
        let fp = build("R", &["weird:id"]);
        assert_eq!(fp, "R:weird-id");
    }

    #[test]
    fn fingerprints_are_deterministic_across_calls() {
        let a = build("R", &["t", "x"]);
        let b = build("R", &["t", "x"]);
        assert_eq!(a, b);
    }
}
