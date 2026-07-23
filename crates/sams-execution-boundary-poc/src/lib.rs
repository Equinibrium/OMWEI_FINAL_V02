//! SAMS Execution Integrity Boundary PoC
//! Demonstrates a deterministic State Transition Integrity Boundary
//!
//! Core concept: Observation Closure
//! An execution is deterministic if and only if every observable input
//! influencing the evaluation is explicitly captured within the EvaluationSnapshot.
//!
//! Formal property:
//! same_claim + same_snapshot + same_predicate_implementation = same_decision

use omwei_atom::Atom;
use std::collections::HashMap;
use std::fmt;
use sha2::{Sha256, Digest};

// ========== TRANSITION PROPOSAL ==========

#[derive(Debug, Clone)]
pub enum RequestedTransition {
    TransferFunds { amount: u64, recipient: String },
    UpdateConfiguration { key: String, value: String },
    ExecuteCommand { command: String, args: Vec<String> },
}

// ========== DESCRIPTORS (Versioning & Identification) ==========

/// PredicateDescriptor: Identifies predicate implementation and version
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PredicateDescriptor {
    pub predicate_id: String,      // "financial_limit_v1" | "robot_safety_check" | ...
    pub version: u32,              // Semantic version number
    pub implementation_hash: [u8; 32],  // SHA256 of predicate bytecode
}

impl PredicateDescriptor {
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.predicate_id.as_bytes());
        hasher.update(self.version.to_le_bytes());
        hasher.update(self.implementation_hash);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

/// BoundaryDescriptor: Identifies boundary implementation and routing semantics
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BoundaryDescriptor {
    pub semantic_version: String,   // "1.0.0" | "2.0.0" | ...
    pub implementation_hash: [u8; 32],  // SHA256 of boundary code
    pub routing_semantics_version: u32,  // For future Bit 31 interpretation changes
}

impl BoundaryDescriptor {
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.semantic_version.as_bytes());
        hasher.update(self.implementation_hash);
        hasher.update(self.routing_semantics_version.to_le_bytes());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

// ========== EXECUTION INTEGRITY CLAIM ==========

/// ExecutionIntegrityClaim: The autonomous decision/intent
/// Immutable once created. Contains the proposed transition and routing bit.
#[derive(Debug, Clone)]
pub struct ExecutionIntegrityClaim {
    pub requested_transition: RequestedTransition,
    pub actor_ref: String,
    pub evidence_refs: Vec<[u8; 32]>,
    pub constraints: TransitionConstraints,
    pub verification_path_bit: bool,
}

impl ExecutionIntegrityClaim {
    pub fn new(
        requested_transition: RequestedTransition,
        actor_ref: String,
        evidence_refs: Vec<[u8; 32]>,
        constraints: TransitionConstraints,
        primary_atom: Atom,
    ) -> Self {
        let verification_path_bit = (primary_atom.global_id & 0x80000000) != 0;
        Self {
            requested_transition,
            actor_ref,
            evidence_refs,
            constraints,
            verification_path_bit,
        }
    }

    /// Canonical hash of claim (deterministic)
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", self.requested_transition).as_bytes());
        hasher.update(self.actor_ref.as_bytes());
        for ev in &self.evidence_refs {
            hasher.update(ev);
        }
        hasher.update(format!("{:?}", self.constraints).as_bytes());
        hasher.update(if self.verification_path_bit { &[1u8] } else { &[0u8] });
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

// ========== OBSERVATION CLOSURE ==========

/// ReferencedState: What system state was observed during evaluation
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReferencedState {
    pub state_id: String,       // "account_ledger" | "robot_position" | "deployment_state"
    pub state_hash: [u8; 32],   // Content address of state snapshot
}

/// EvidenceReference: What external evidence was available
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EvidenceReference {
    pub evidence_hash: [u8; 32],    // Content address of evidence
    pub evidence_type: String,      // "market_snapshot" | "compliance_check" | "sensor_frame"
    pub acquired_at: u64,           // Timestamp when evidence was acquired
}

/// EvaluationSnapshot: The complete deterministic closure
/// Contains all observable inputs the predicate was allowed to observe.
/// If two snapshots have identical hashes, their predicates MUST produce identical results.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EvaluationSnapshot {
    pub predicate: PredicateDescriptor,
    pub boundary: BoundaryDescriptor,
    pub referenced_states: Vec<ReferencedState>,
    pub evidence_references: Vec<EvidenceReference>,
    pub evaluation_timestamp: u64,  // Metadata for audit trail (not part of determinism)
}

impl EvaluationSnapshot {
    /// Canonical hash of observation closure (deterministic)
    /// This hash IS the observation closure.
    /// If hash(S1) == hash(S2), then P(S1) == P(S2) for any pure predicate P.
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.predicate.compute_hash());
        hasher.update(self.boundary.compute_hash());
        for state in &self.referenced_states {
            hasher.update(state.state_id.as_bytes());
            hasher.update(state.state_hash);
        }
        for evidence in &self.evidence_references {
            hasher.update(evidence.evidence_hash);
            hasher.update(evidence.evidence_type.as_bytes());
            hasher.update(evidence.acquired_at.to_le_bytes());
        }
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Check if two snapshots represent identical observations
    pub fn is_observation_equivalent(&self, other: &EvaluationSnapshot) -> bool {
        self.compute_hash() == other.compute_hash()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TransitionConstraints {
    pub max_amount: Option<u64>,
    pub time_window_start: Option<u64>,
    pub time_window_end: Option<u64>,
    pub required_evidence_count: usize,
}

impl Default for TransitionConstraints {
    fn default() -> Self {
        Self {
            max_amount: Some(50_000),
            time_window_start: None,
            time_window_end: None,
            required_evidence_count: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredicateResult {
    Approved,
    Rejected,
    IncompleteEvidence { missing: Vec<[u8; 32]> },
}

impl fmt::Display for PredicateResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PredicateResult::Approved => write!(f, "APPROVED"),
            PredicateResult::Rejected => write!(f, "REJECTED"),
            PredicateResult::IncompleteEvidence { missing } => write!(f, "INCOMPLETE_EVIDENCE ({} missing)", missing.len()),
        }
    }
}

// ========== PREDICATE ERROR ==========

#[derive(Debug, Clone, thiserror::Error)]
pub enum PredicateError {
    #[error("Predicate descriptor mismatch")]
    DescriptorMismatch,
    #[error("Unsupported transition type: {0}")]
    UnsupportedTransitionType(String),
    #[error("State reference not found: {0}")]
    StateNotFound(String),
    #[error("Evidence reference not found: {0}")]
    EvidenceNotFound(String),
    #[error("Internal predicate error: {0}")]
    InternalError(String),
}

// ========== PREDICATE TRAIT ==========

/// TransitionPredicate: Pure function over observation closure
/// INVARIANT: Must be side-effect free. Reads only from snapshot and state.
/// INVARIANT: Identical snapshots must produce identical results.
pub trait TransitionPredicate: Send + Sync {
    fn descriptor(&self) -> PredicateDescriptor;
    fn evaluate(
        &self,
        claim: &ExecutionIntegrityClaim,
        snapshot: &EvaluationSnapshot,
        current_state: &SystemState,
    ) -> Result<PredicateResult, PredicateError>;
}

// ========== PREDICATE IMPLEMENTATIONS ==========

#[derive(Debug, Clone)]
pub struct FinancialLimitPredicate {
    pub max_transfer_amount: u64,
    pub require_invoice_evidence: bool,
}

impl FinancialLimitPredicate {
    pub fn new(max_transfer_amount: u64, require_invoice_evidence: bool) -> Self {
        Self {
            max_transfer_amount,
            require_invoice_evidence,
        }
    }

    pub fn implementation_hash() -> [u8; 32] {
        // In production: SHA256 of compiled bytecode
        // For PoC: deterministic hash of logic
        let mut hash = [0u8; 32];
        hash[0] = 0xf1;  // Financial predicate marker
        hash
    }
}

impl TransitionPredicate for FinancialLimitPredicate {
    fn descriptor(&self) -> PredicateDescriptor {
        PredicateDescriptor {
            predicate_id: "financial_limit_predicate".to_string(),
            version: 1,
            implementation_hash: Self::implementation_hash(),
        }
    }

    fn evaluate(
        &self,
        claim: &ExecutionIntegrityClaim,
        snapshot: &EvaluationSnapshot,
        _current_state: &SystemState,
    ) -> Result<PredicateResult, PredicateError> {
        // Verify predicate descriptor matches
        if snapshot.predicate != self.descriptor() {
            return Err(PredicateError::DescriptorMismatch);
        }

        match &claim.requested_transition {
            RequestedTransition::TransferFunds { amount, .. } => {
                if *amount > self.max_transfer_amount {
                    return Ok(PredicateResult::Rejected);
                }
                if self.require_invoice_evidence && claim.evidence_refs.is_empty() {
                    let missing = claim.evidence_refs.clone();
                    return Ok(PredicateResult::IncompleteEvidence { missing });
                }
                Ok(PredicateResult::Approved)
            }
            _ => Err(PredicateError::UnsupportedTransitionType(
                format!("{:?}", claim.requested_transition)
            )),
        }
    }
}

// ========== SYSTEM STATE ==========

#[derive(Debug, Clone)]
pub struct SystemState {
    pub account_balances: HashMap<String, u64>,
    pub configuration: HashMap<String, String>,
    pub last_transition_id: Option<String>,
}

impl SystemState {
    pub fn new() -> Self {
        Self {
            account_balances: HashMap::new(),
            configuration: HashMap::new(),
            last_transition_id: None,
        }
    }

    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for (key, value) in &self.account_balances {
            hasher.update(key.as_bytes());
            hasher.update(value.to_le_bytes());
        }
        for (key, value) in &self.configuration {
            hasher.update(key.as_bytes());
            hasher.update(value.as_bytes());
        }
        if let Some(id) = &self.last_transition_id {
            hasher.update(id.as_bytes());
        }
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

impl Default for SystemState {
    fn default() -> Self {
        Self::new()
    }
}

// ========== BOUNDARY ERRORS ==========

#[derive(Debug, Clone, thiserror::Error)]
pub enum BoundaryError {
    #[error("Extended verification required (verification_path_bit = 1)")]
    ExtendedVerificationRequired,
    #[error("Predicate evaluation rejected")]
    PredicateRejected,
    #[error("Incomplete evidence: {0}")]
    IncompleteEvidence(String),
    #[error("Predicate error: {0}")]
    PredicateError(String),
    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),
    #[error("Determinism violation: snapshot_hash mismatch or non-deterministic predicate behavior")]
    DeterminismViolation,
}

// ========== DETERMINISM VERIFIER ==========

pub struct DeterminismVerifier;

impl DeterminismVerifier {
    /// Verify that two snapshots are observation-equivalent
    pub fn are_observation_equivalent(
        snapshot1: &EvaluationSnapshot,
        snapshot2: &EvaluationSnapshot,
    ) -> bool {
        snapshot1.is_observation_equivalent(snapshot2)
    }

    /// Verify that identical snapshots produce identical predicate results
    pub fn verify_determinism(
        predicate: &dyn TransitionPredicate,
        claim: &ExecutionIntegrityClaim,
        snapshot: &EvaluationSnapshot,
        state: &SystemState,
    ) -> Result<PredicateResult, PredicateError> {
        // Run predicate once
        let result1 = predicate.evaluate(claim, snapshot, state)?;

        // Run again with identical inputs
        let result2 = predicate.evaluate(claim, snapshot, state)?;

        // Results must be identical
        if result1 != result2 {
            return Err(PredicateError::InternalError(
                "Determinism violation: identical inputs produced different results".to_string()
            ));
        }

        Ok(result1)
    }

    /// Check if two records represent identical execution
    pub fn are_records_deterministic_equivalent(
        record1: &CausalAccountabilityRecord,
        record2: &CausalAccountabilityRecord,
    ) -> bool {
        record1.snapshot_hash == record2.snapshot_hash
            && record1.predicate_result == record2.predicate_result
    }
}

// ========== EXECUTION BOUNDARY GATE ==========

pub struct ExecutionBoundaryGate {
    predicate: Box<dyn TransitionPredicate>,
    boundary_descriptor: BoundaryDescriptor,
}

impl ExecutionBoundaryGate {
    pub fn new(predicate: Box<dyn TransitionPredicate>, boundary_descriptor: BoundaryDescriptor) -> Self {
        Self { predicate, boundary_descriptor }
    }

    pub fn process_transition(
        &self,
        claim: ExecutionIntegrityClaim,
        snapshot: EvaluationSnapshot,
        state: &mut SystemState,
    ) -> Result<CausalAccountabilityRecord, BoundaryError> {
        // Step 1: Verify snapshot consistency
        self.verify_snapshot_consistency(&snapshot)?;

        // Step 2: Route based on verification_path_bit
        if claim.verification_path_bit {
            return Err(BoundaryError::ExtendedVerificationRequired);
        }

        // Step 3: Evaluate predicate (pure function)
        let state_before_hash = state.compute_hash();
        let predicate_result = self.predicate.evaluate(&claim, &snapshot, state)
            .map_err(|e| BoundaryError::PredicateError(e.to_string()))?;

        // Step 4: Conditional state mutation
        match predicate_result {
            PredicateResult::Approved => {
                self.apply_transition(&claim, state)?;
                let state_after_hash = state.compute_hash();

                let record = CausalAccountabilityRecord {
                    claim_hash: claim.compute_hash(),
                    snapshot_hash: snapshot.compute_hash(),
                    predicate_result: PredicateResult::Approved,
                    state_before_hash,
                    state_after_hash,
                    actor_ref: claim.actor_ref.clone(),
                    transition_type: format!("{:?}", claim.requested_transition),
                    boundary_version: self.boundary_descriptor.semantic_version.clone(),
                };

                Ok(record)
            }
            PredicateResult::Rejected => {
                Err(BoundaryError::PredicateRejected)
            }
            PredicateResult::IncompleteEvidence { missing } => {
                let msg = format!("{} pieces of evidence missing", missing.len());
                Err(BoundaryError::IncompleteEvidence(msg))
            }
        }
    }

    fn verify_snapshot_consistency(&self, snapshot: &EvaluationSnapshot) -> Result<(), BoundaryError> {
        // Verify that snapshot's boundary descriptor matches ours
        if snapshot.boundary != self.boundary_descriptor {
            return Err(BoundaryError::PredicateError(
                "Snapshot boundary descriptor mismatch".to_string()
            ));
        }
        Ok(())
    }

    fn apply_transition(&self, claim: &ExecutionIntegrityClaim, state: &mut SystemState) -> Result<(), BoundaryError> {
        match &claim.requested_transition {
            RequestedTransition::TransferFunds { amount, recipient } => {
                let current_balance = state.account_balances.get(recipient).copied().unwrap_or(0);
                state.account_balances.insert(recipient.clone(), current_balance + amount);
                Ok(())
            }
            RequestedTransition::UpdateConfiguration { key, value } => {
                state.configuration.insert(key.clone(), value.clone());
                Ok(())
            }
            RequestedTransition::ExecuteCommand { .. } => {
                Err(BoundaryError::InvalidTransition(
                    "Command execution not implemented in PoC".to_string()
                ))
            }
        }
    }
}

// ========== CAUSAL ACCOUNTABILITY RECORD ==========

/// CausalAccountabilityRecord: Immutable audit entry
/// Contains the observation closure and decision, sufficient for replay and verification.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CausalAccountabilityRecord {
    pub claim_hash: [u8; 32],
    pub snapshot_hash: [u8; 32],           // Observation closure
    pub predicate_result: PredicateResult,
    pub state_before_hash: [u8; 32],
    pub state_after_hash: [u8; 32],
    pub actor_ref: String,
    pub transition_type: String,
    pub boundary_version: String,
}

impl fmt::Display for CausalAccountabilityRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== Causal Accountability Record ===")?;
        writeln!(f, "Observation Closure: {}", hex(&self.snapshot_hash[..8]))?;
        writeln!(f, "Predicate Result:    {}", self.predicate_result)?;
        writeln!(f, "Actor:               {}", self.actor_ref)?;
        writeln!(f, "Transition Type:     {}", self.transition_type)?;
        writeln!(f, "Boundary Version:    {}", self.boundary_version)?;
        Ok(())
    }
}

// ========== TEST UTILITIES ==========

pub fn create_test_atom(global_id: u32) -> Atom {
    Atom {
        global_id,
        payload: [0u8; 28],
    }
}

pub fn create_test_evidence(seed: u8) -> [u8; 32] {
    let mut evidence = [0u8; 32];
    for i in 0..32 {
        evidence[i] = seed.wrapping_add(i as u8);
    }
    evidence
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn create_boundary_descriptor() -> BoundaryDescriptor {
    BoundaryDescriptor {
        semantic_version: "1.0.0".to_string(),
        implementation_hash: {
            let mut hash = [0u8; 32];
            hash[0] = 0xb0;  // Boundary marker
            hash
        },
        routing_semantics_version: 1,
    }
}

pub fn create_evaluation_snapshot(
    predicate: PredicateDescriptor,
    boundary: BoundaryDescriptor,
    state_hash: [u8; 32],
    evidence_hashes: Vec<[u8; 32]>,
) -> EvaluationSnapshot {
    let mut evidence_references = Vec::new();
    for (idx, hash) in evidence_hashes.iter().enumerate() {
        evidence_references.push(EvidenceReference {
            evidence_hash: *hash,
            evidence_type: format!("test_evidence_{}", idx),
            acquired_at: 1000 + idx as u64,
        });
    }

    EvaluationSnapshot {
        predicate,
        boundary,
        referenced_states: vec![ReferencedState {
            state_id: "account_ledger".to_string(),
            state_hash,
        }],
        evidence_references,
        evaluation_timestamp: 1000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determinism_identical_snapshots() {
        let predicate = FinancialLimitPredicate::new(50_000, true);
        let mut state = SystemState::new();
        state.account_balances.insert("recipient@example.com".to_string(), 100_000);

        let claim = ExecutionIntegrityClaim::new(
            RequestedTransition::TransferFunds {
                amount: 10_000,
                recipient: "recipient@example.com".to_string(),
            },
            "agent-test".to_string(),
            vec![create_test_evidence(1)],
            TransitionConstraints::default(),
            create_test_atom(0x00000001),
        );

        let boundary = create_boundary_descriptor();
        let snapshot1 = create_evaluation_snapshot(
            predicate.descriptor(),
            boundary.clone(),
            state.compute_hash(),
            vec![create_test_evidence(1)],
        );
        let snapshot2 = snapshot1.clone();

        // Both snapshots must have identical hashes
        assert_eq!(snapshot1.compute_hash(), snapshot2.compute_hash());

        // Predicate must produce identical results
        let result1 = predicate.evaluate(&claim, &snapshot1, &state).unwrap();
        let result2 = predicate.evaluate(&claim, &snapshot2, &state).unwrap();
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_state_mutation_on_predicate_success() {
        let mut state = SystemState::new();
        state.account_balances.insert("recipient@example.com".to_string(), 100_000);

        let claim = ExecutionIntegrityClaim::new(
            RequestedTransition::TransferFunds {
                amount: 10_000,
                recipient: "recipient@example.com".to_string(),
            },
            "agent-test".to_string(),
            vec![create_test_evidence(1)],
            TransitionConstraints::default(),
            create_test_atom(0x00000001),
        );

        let predicate = FinancialLimitPredicate::new(50_000, true);
        let boundary = create_boundary_descriptor();
        let snapshot = create_evaluation_snapshot(
            predicate.descriptor(),
            boundary.clone(),
            state.compute_hash(),
            vec![create_test_evidence(1)],
        );

        let gate = ExecutionBoundaryGate::new(
            Box::new(predicate),
            boundary,
        );

        let result = gate.process_transition(claim, snapshot, &mut state);

        assert!(result.is_ok());
        assert_eq!(state.account_balances.get("recipient@example.com"), Some(&110_000));
    }

    #[test]
    fn test_verification_path_bit_extraction() {
        let atom_inline = create_test_atom(0x00000001);
        let claim_inline = ExecutionIntegrityClaim::new(
            RequestedTransition::TransferFunds {
                amount: 10_000,
                recipient: "test@example.com".to_string(),
            },
            "agent-test".to_string(),
            vec![],
            TransitionConstraints::default(),
            atom_inline,
        );
        assert_eq!(claim_inline.verification_path_bit, false);

        let atom_extended = create_test_atom(0x80000001);
        let claim_extended = ExecutionIntegrityClaim::new(
            RequestedTransition::TransferFunds {
                amount: 10_000,
                recipient: "test@example.com".to_string(),
            },
            "agent-test".to_string(),
            vec![],
            TransitionConstraints::default(),
            atom_extended,
        );
        assert_eq!(claim_extended.verification_path_bit, true);
    }

    #[test]
    fn test_observation_closure_equivalence() {
        let predicate = FinancialLimitPredicate::new(50_000, true);
        let boundary = create_boundary_descriptor();
        let state = SystemState::new();

        let snapshot1 = create_evaluation_snapshot(
            predicate.descriptor(),
            boundary.clone(),
            state.compute_hash(),
            vec![create_test_evidence(1), create_test_evidence(2)],
        );

        let snapshot2 = create_evaluation_snapshot(
            predicate.descriptor(),
            boundary.clone(),
            state.compute_hash(),
            vec![create_test_evidence(1), create_test_evidence(2)],
        );

        // Same observations = same closure hash
        assert!(snapshot1.is_observation_equivalent(&snapshot2));
    }
}
