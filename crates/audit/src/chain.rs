use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::canonical::to_canonical_json;

const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Events that can be recorded in the audit log.
///
/// Each variant captures the minimum stable fields needed to reconstruct and
/// verify what happened. No timestamps — ordering is guaranteed by the chain's
/// sequence number and hash linkage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum AuditEvent {
    CaseRouted {
        case_id: String,
        proof_hash: String,
        selected_candidate_id: String,
    },
    CaseRefused {
        case_id: String,
        proof_hash: String,
        refusal_code: String,
    },
}

/// Payload hashed for each entry (excludes the `hash` field itself).
#[derive(Serialize)]
struct EntryPayload<'a> {
    seq: u64,
    event: &'a AuditEvent,
    previous_hash: &'a str,
}

/// One immutable record in the audit log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditEntry {
    pub seq: u64,
    pub event: AuditEvent,
    pub previous_hash: String,
    pub hash: String,
}

impl AuditEntry {
    fn new(seq: u64, event: AuditEvent, previous_hash: String) -> Self {
        let payload = EntryPayload {
            seq,
            event: &event,
            previous_hash: &previous_hash,
        };
        let canonical = to_canonical_json(&payload);
        let hash = sha256_hex(&canonical);
        Self {
            seq,
            event,
            previous_hash,
            hash,
        }
    }
}

/// Hash-chained, append-only audit log.
///
/// Each entry's `hash` is SHA-256 of the canonical JSON of
/// `{ seq, event, previous_hash }`. The first entry's `previous_hash` is the
/// 64-zero genesis hash. `verify_chain` recomputes every hash and checks
/// linkage — any mutation or reordering of entries causes verification to fail.
#[derive(Debug, Default)]
pub struct AuditLog {
    entries: Vec<AuditEntry>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an event and returns a reference to the newly created entry.
    pub fn append(&mut self, event: AuditEvent) -> &AuditEntry {
        let seq = self.entries.len() as u64;
        let previous_hash = self
            .entries
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(|| GENESIS_HASH.to_string());
        let entry = AuditEntry::new(seq, event, previous_hash);
        self.entries.push(entry);
        self.entries.last().unwrap()
    }

    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Recomputes every entry's hash and checks hash linkage.
    ///
    /// Returns `true` if the chain is intact, `false` if any entry has been
    /// mutated or entries are out of order.
    pub fn verify_chain(&self) -> bool {
        for (i, entry) in self.entries.iter().enumerate() {
            let expected_previous = if i == 0 {
                GENESIS_HASH.to_string()
            } else {
                self.entries[i - 1].hash.clone()
            };

            if entry.previous_hash != expected_previous {
                return false;
            }

            let payload = EntryPayload {
                seq: entry.seq,
                event: &entry.event,
                previous_hash: &entry.previous_hash,
            };
            let canonical = to_canonical_json(&payload);
            if sha256_hex(&canonical) != entry.hash {
                return false;
            }
        }
        true
    }
}

fn sha256_hex(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    format!("{:x}", digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn routed(case_id: &str, proof_hash: &str, candidate_id: &str) -> AuditEvent {
        AuditEvent::CaseRouted {
            case_id: case_id.to_string(),
            proof_hash: proof_hash.to_string(),
            selected_candidate_id: candidate_id.to_string(),
        }
    }

    fn refused(case_id: &str, proof_hash: &str, code: &str) -> AuditEvent {
        AuditEvent::CaseRefused {
            case_id: case_id.to_string(),
            proof_hash: proof_hash.to_string(),
            refusal_code: code.to_string(),
        }
    }

    // ── empty log ─────────────────────────────────────────────────────────────

    #[test]
    fn empty_log_verifies() {
        let log = AuditLog::new();
        assert!(log.verify_chain());
    }

    #[test]
    fn empty_log_has_zero_entries() {
        let log = AuditLog::new();
        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
    }

    // ── single entry ──────────────────────────────────────────────────────────

    #[test]
    fn first_entry_has_seq_zero_and_genesis_previous_hash() {
        let mut log = AuditLog::new();
        log.append(routed("case-1", "aabbcc", "rc-1"));
        let entry = &log.entries()[0];
        assert_eq!(entry.seq, 0);
        assert_eq!(entry.previous_hash, GENESIS_HASH);
    }

    #[test]
    fn single_entry_log_verifies() {
        let mut log = AuditLog::new();
        log.append(routed("case-1", "aabbcc", "rc-1"));
        assert!(log.verify_chain());
    }

    #[test]
    fn entry_hash_is_64_hex_chars() {
        let mut log = AuditLog::new();
        log.append(routed("case-1", "aabbcc", "rc-1"));
        let hash = &log.entries()[0].hash;
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ── multiple entries ──────────────────────────────────────────────────────

    #[test]
    fn second_entry_previous_hash_matches_first_entry_hash() {
        let mut log = AuditLog::new();
        log.append(routed("case-1", "aabbcc", "rc-1"));
        log.append(refused("case-2", "ddeeff", "no_eligible_candidates"));
        let hash_0 = log.entries()[0].hash.clone();
        let prev_1 = log.entries()[1].previous_hash.clone();
        assert_eq!(hash_0, prev_1);
    }

    #[test]
    fn sequence_numbers_are_monotonically_increasing() {
        let mut log = AuditLog::new();
        log.append(routed("case-1", "aabbcc", "rc-1"));
        log.append(routed("case-2", "112233", "rc-2"));
        log.append(refused("case-3", "445566", "compliance_failed"));
        for (i, entry) in log.entries().iter().enumerate() {
            assert_eq!(entry.seq, i as u64);
        }
    }

    #[test]
    fn three_entry_log_verifies() {
        let mut log = AuditLog::new();
        log.append(routed("case-1", "aabbcc", "rc-1"));
        log.append(refused("case-2", "ddeeff", "no_eligible_candidates"));
        log.append(routed("case-3", "001122", "rc-3"));
        assert!(log.verify_chain());
    }

    // ── determinism ───────────────────────────────────────────────────────────

    #[test]
    fn same_events_produce_identical_hashes() {
        let mut log_a = AuditLog::new();
        log_a.append(routed("case-1", "aabbcc", "rc-1"));
        log_a.append(refused("case-2", "ddeeff", "no_eligible_candidates"));

        let mut log_b = AuditLog::new();
        log_b.append(routed("case-1", "aabbcc", "rc-1"));
        log_b.append(refused("case-2", "ddeeff", "no_eligible_candidates"));

        assert_eq!(log_a.entries()[0].hash, log_b.entries()[0].hash);
        assert_eq!(log_a.entries()[1].hash, log_b.entries()[1].hash);
    }

    #[test]
    fn different_events_produce_different_hashes() {
        let mut log_a = AuditLog::new();
        log_a.append(routed("case-1", "aabbcc", "rc-1"));

        let mut log_b = AuditLog::new();
        log_b.append(routed("case-1", "aabbcc", "rc-2")); // different candidate

        assert_ne!(log_a.entries()[0].hash, log_b.entries()[0].hash);
    }

    // ── tamper detection ──────────────────────────────────────────────────────

    #[test]
    fn mutating_entry_hash_fails_verify() {
        let mut log = AuditLog::new();
        log.append(routed("case-1", "aabbcc", "rc-1"));
        log.entries.first_mut().unwrap().hash =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        assert!(!log.verify_chain());
    }

    #[test]
    fn mutating_previous_hash_fails_verify() {
        let mut log = AuditLog::new();
        log.append(routed("case-1", "aabbcc", "rc-1"));
        log.append(routed("case-2", "112233", "rc-2"));
        log.entries[1].previous_hash =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        assert!(!log.verify_chain());
    }

    #[test]
    fn mutating_event_field_fails_verify() {
        let mut log = AuditLog::new();
        log.append(routed("case-1", "aabbcc", "rc-1"));
        // Swap the event to a different one with the same shape.
        log.entries[0].event = routed("case-TAMPERED", "aabbcc", "rc-1");
        assert!(!log.verify_chain());
    }

    #[test]
    fn swapping_entry_order_fails_verify() {
        let mut log = AuditLog::new();
        log.append(routed("case-1", "aabbcc", "rc-1"));
        log.append(refused("case-2", "ddeeff", "no_eligible_candidates"));
        log.entries.swap(0, 1);
        assert!(!log.verify_chain());
    }

    // ── append return value ───────────────────────────────────────────────────

    #[test]
    fn append_returns_reference_to_appended_entry() {
        let mut log = AuditLog::new();
        let entry = log.append(routed("case-1", "aabbcc", "rc-1"));
        assert_eq!(entry.seq, 0);
        assert_eq!(entry.previous_hash, GENESIS_HASH);
    }

    // ── event serialisation ───────────────────────────────────────────────────

    #[test]
    fn case_routed_event_canonical_json_contains_event_type() {
        let event = routed("case-1", "aabbcc", "rc-1");
        let json = to_canonical_json(&event);
        assert!(json.contains("\"event_type\":\"case_routed\""));
    }

    #[test]
    fn case_refused_event_canonical_json_contains_event_type() {
        let event = refused("case-1", "aabbcc", "compliance_failed");
        let json = to_canonical_json(&event);
        assert!(json.contains("\"event_type\":\"case_refused\""));
    }
}
