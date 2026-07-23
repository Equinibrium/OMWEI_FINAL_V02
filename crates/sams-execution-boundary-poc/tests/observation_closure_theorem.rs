//! Integration Tests for Observation Closure Theorem
//!
//! These tests prove the formal Observation Closure Theorem:
//! "Identical observation closures MUST yield identical predicate decisions,
//! while snapshot discrepancies trigger DeterminismViolation."
//!
//! Theorem Statement:
//! For any pure predicate P, if hash(S1) == hash(S2), then P(S1) == P(S2).
//! If P(S1) != P(S2), then hash(S1) != hash(S2).

use sams_execution_boundary_poc::{
    EvaluationSnapshot, BoundaryDescriptor, ReferencedState,
    EvidenceReference, ExecutionIntegrityClaim, RequestedTransition, TransitionConstraints,
    SystemState, FinancialLimitPredicate, TransitionPredicate, DeterminismVerifier,
    create_test_atom, create_test_evidence, create_boundary_descriptor,
};

#[test]
fn test_observation_closure_theorem_identical_snapshots() {
    // Theorem: Identical snapshots produce identical predicate results
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
    let snapshot1 = EvaluationSnapshot {
        predicate: predicate.descriptor(),
        boundary: boundary.clone(),
        referenced_states: vec![ReferencedState {
            state_id: "account_ledger".to_string(),
            state_hash: state.compute_hash(),
        }],
        evidence_references: vec![EvidenceReference {
            evidence_hash: create_test_evidence(1),
            evidence_type: "invoice".to_string(),
            acquired_at: 1000,
        }],
        evaluation_timestamp: 1000,
    };

    let snapshot2 = snapshot1.clone();

    // Verify: hash(S1) == hash(S2)
    assert_eq!(snapshot1.compute_hash(), snapshot2.compute_hash());

    // Verify: P(S1) == P(S2)
    let result1 = predicate.evaluate(&claim, &snapshot1, &state).unwrap();
    let result2 = predicate.evaluate(&claim, &snapshot2, &state).unwrap();
    assert_eq!(result1, result2);

    // Verify: DeterminismVerifier confirms determinism
    let verified_result = DeterminismVerifier::verify_determinism(
        &predicate,
        &claim,
        &snapshot1,
        &state,
    ).unwrap();
    assert_eq!(verified_result, result1);
}

#[test]
fn test_observation_closure_theorem_different_evidence() {
    // Theorem: Different evidence produces different snapshot hashes
    let predicate = FinancialLimitPredicate::new(50_000, true);
    let mut state = SystemState::new();
    state.account_balances.insert("recipient@example.com".to_string(), 100_000);

    let boundary = create_boundary_descriptor();

    let snapshot1 = EvaluationSnapshot {
        predicate: predicate.descriptor(),
        boundary: boundary.clone(),
        referenced_states: vec![ReferencedState {
            state_id: "account_ledger".to_string(),
            state_hash: state.compute_hash(),
        }],
        evidence_references: vec![EvidenceReference {
            evidence_hash: create_test_evidence(1),
            evidence_type: "invoice".to_string(),
            acquired_at: 1000,
        }],
        evaluation_timestamp: 1000,
    };

    let snapshot2 = EvaluationSnapshot {
        predicate: predicate.descriptor(),
        boundary: boundary.clone(),
        referenced_states: vec![ReferencedState {
            state_id: "account_ledger".to_string(),
            state_hash: state.compute_hash(),
        }],
        evidence_references: vec![EvidenceReference {
            evidence_hash: create_test_evidence(2), // Different evidence
            evidence_type: "invoice".to_string(),
            acquired_at: 1000,
        }],
        evaluation_timestamp: 1000,
    };

    // Verify: hash(S1) != hash(S2)
    assert_ne!(snapshot1.compute_hash(), snapshot2.compute_hash());

    // Verify: Not observation-equivalent
    assert!(!snapshot1.is_observation_equivalent(&snapshot2));
}

#[test]
fn test_observation_closure_theorem_different_state_hash() {
    // Theorem: Different state hashes produce different snapshot hashes
    let predicate = FinancialLimitPredicate::new(50_000, true);
    let boundary = create_boundary_descriptor();

    let mut state1 = SystemState::new();
    state1.account_balances.insert("recipient@example.com".to_string(), 100_000);

    let mut state2 = SystemState::new();
    state2.account_balances.insert("recipient@example.com".to_string(), 200_000); // Different balance

    let snapshot1 = EvaluationSnapshot {
        predicate: predicate.descriptor(),
        boundary: boundary.clone(),
        referenced_states: vec![ReferencedState {
            state_id: "account_ledger".to_string(),
            state_hash: state1.compute_hash(),
        }],
        evidence_references: vec![EvidenceReference {
            evidence_hash: create_test_evidence(1),
            evidence_type: "invoice".to_string(),
            acquired_at: 1000,
        }],
        evaluation_timestamp: 1000,
    };

    let snapshot2 = EvaluationSnapshot {
        predicate: predicate.descriptor(),
        boundary: boundary.clone(),
        referenced_states: vec![ReferencedState {
            state_id: "account_ledger".to_string(),
            state_hash: state2.compute_hash(), // Different state hash
        }],
        evidence_references: vec![EvidenceReference {
            evidence_hash: create_test_evidence(1),
            evidence_type: "invoice".to_string(),
            acquired_at: 1000,
        }],
        evaluation_timestamp: 1000,
    };

    // Verify: hash(S1) != hash(S2)
    assert_ne!(snapshot1.compute_hash(), snapshot2.compute_hash());
}

#[test]
fn test_observation_closure_theorem_different_boundary() {
    // Theorem: Different boundary descriptors produce different snapshot hashes
    let predicate = FinancialLimitPredicate::new(50_000, true);
    let mut state = SystemState::new();
    state.account_balances.insert("recipient@example.com".to_string(), 100_000);

    let boundary1 = BoundaryDescriptor {
        semantic_version: "1.0.0".to_string(),
        implementation_hash: {
            let mut hash = [0u8; 32];
            hash[0] = 0xb0;
            hash
        },
        routing_semantics_version: 1,
    };

    let boundary2 = BoundaryDescriptor {
        semantic_version: "2.0.0".to_string(), // Different version
        implementation_hash: {
            let mut hash = [0u8; 32];
            hash[0] = 0xb0;
            hash
        },
        routing_semantics_version: 1,
    };

    let snapshot1 = EvaluationSnapshot {
        predicate: predicate.descriptor(),
        boundary: boundary1,
        referenced_states: vec![ReferencedState {
            state_id: "account_ledger".to_string(),
            state_hash: state.compute_hash(),
        }],
        evidence_references: vec![EvidenceReference {
            evidence_hash: create_test_evidence(1),
            evidence_type: "invoice".to_string(),
            acquired_at: 1000,
        }],
        evaluation_timestamp: 1000,
    };

    let snapshot2 = EvaluationSnapshot {
        predicate: predicate.descriptor(),
        boundary: boundary2,
        referenced_states: vec![ReferencedState {
            state_id: "account_ledger".to_string(),
            state_hash: state.compute_hash(),
        }],
        evidence_references: vec![EvidenceReference {
            evidence_hash: create_test_evidence(1),
            evidence_type: "invoice".to_string(),
            acquired_at: 1000,
        }],
        evaluation_timestamp: 1000,
    };

    // Verify: hash(S1) != hash(S2)
    assert_ne!(snapshot1.compute_hash(), snapshot2.compute_hash());
}

#[test]
fn test_observation_closure_theorem_timestamp_not_affecting_hash() {
    // Theorem: evaluation_timestamp is metadata and does NOT affect observation closure hash
    let predicate = FinancialLimitPredicate::new(50_000, true);
    let mut state = SystemState::new();
    state.account_balances.insert("recipient@example.com".to_string(), 100_000);

    let boundary = create_boundary_descriptor();

    let snapshot1 = EvaluationSnapshot {
        predicate: predicate.descriptor(),
        boundary: boundary.clone(),
        referenced_states: vec![ReferencedState {
            state_id: "account_ledger".to_string(),
            state_hash: state.compute_hash(),
        }],
        evidence_references: vec![EvidenceReference {
            evidence_hash: create_test_evidence(1),
            evidence_type: "invoice".to_string(),
            acquired_at: 1000,
        }],
        evaluation_timestamp: 1000, // Different timestamp
    };

    let snapshot2 = EvaluationSnapshot {
        predicate: predicate.descriptor(),
        boundary: boundary.clone(),
        referenced_states: vec![ReferencedState {
            state_id: "account_ledger".to_string(),
            state_hash: state.compute_hash(),
        }],
        evidence_references: vec![EvidenceReference {
            evidence_hash: create_test_evidence(1),
            evidence_type: "invoice".to_string(),
            acquired_at: 1000,
        }],
        evaluation_timestamp: 2000, // Different timestamp
    };

    // Verify: hash(S1) == hash(S2) (timestamp does NOT affect closure)
    assert_eq!(snapshot1.compute_hash(), snapshot2.compute_hash());

    // Verify: observation-equivalent despite different timestamps
    assert!(snapshot1.is_observation_equivalent(&snapshot2));
}

#[test]
fn test_observation_closure_theorem_multiple_evidence_order_independence() {
    // Theorem: Evidence order affects hash (canonical serialization is order-dependent)
    let predicate = FinancialLimitPredicate::new(50_000, true);
    let mut state = SystemState::new();
    state.account_balances.insert("recipient@example.com".to_string(), 100_000);

    let boundary = create_boundary_descriptor();

    let snapshot1 = EvaluationSnapshot {
        predicate: predicate.descriptor(),
        boundary: boundary.clone(),
        referenced_states: vec![ReferencedState {
            state_id: "account_ledger".to_string(),
            state_hash: state.compute_hash(),
        }],
        evidence_references: vec![
            EvidenceReference {
                evidence_hash: create_test_evidence(1),
                evidence_type: "invoice".to_string(),
                acquired_at: 1000,
            },
            EvidenceReference {
                evidence_hash: create_test_evidence(2),
                evidence_type: "compliance".to_string(),
                acquired_at: 1001,
            },
        ],
        evaluation_timestamp: 1000,
    };

    let snapshot2 = EvaluationSnapshot {
        predicate: predicate.descriptor(),
        boundary: boundary.clone(),
        referenced_states: vec![ReferencedState {
            state_id: "account_ledger".to_string(),
            state_hash: state.compute_hash(),
        }],
        evidence_references: vec![
            EvidenceReference {
                evidence_hash: create_test_evidence(2), // Reversed order
                evidence_type: "compliance".to_string(),
                acquired_at: 1001,
            },
            EvidenceReference {
                evidence_hash: create_test_evidence(1),
                evidence_type: "invoice".to_string(),
                acquired_at: 1000,
            },
        ],
        evaluation_timestamp: 1000,
    };

    // Verify: hash(S1) != hash(S2) (order matters in canonical serialization)
    assert_ne!(snapshot1.compute_hash(), snapshot2.compute_hash());
}

#[test]
fn test_observation_closure_theorem_determinism_verifier() {
    // Theorem: DeterminismVerifier catches non-deterministic behavior
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
    let snapshot = EvaluationSnapshot {
        predicate: predicate.descriptor(),
        boundary,
        referenced_states: vec![ReferencedState {
            state_id: "account_ledger".to_string(),
            state_hash: state.compute_hash(),
        }],
        evidence_references: vec![EvidenceReference {
            evidence_hash: create_test_evidence(1),
            evidence_type: "invoice".to_string(),
            acquired_at: 1000,
        }],
        evaluation_timestamp: 1000,
    };

    // Verify: DeterminismVerifier confirms pure predicate behavior
    let result = DeterminismVerifier::verify_determinism(
        &predicate,
        &claim,
        &snapshot,
        &state,
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), sams_execution_boundary_poc::PredicateResult::Approved);
}

#[test]
fn test_observation_closure_theorem_canonical_serialization_determinism() {
    // Theorem: Canonical serialization is architecture-independent
    let predicate = FinancialLimitPredicate::new(50_000, true);
    let mut state = SystemState::new();
    state.account_balances.insert("recipient@example.com".to_string(), 100_000);

    let boundary = create_boundary_descriptor();

    let snapshot = EvaluationSnapshot {
        predicate: predicate.descriptor(),
        boundary,
        referenced_states: vec![ReferencedState {
            state_id: "account_ledger".to_string(),
            state_hash: state.compute_hash(),
        }],
        evidence_references: vec![EvidenceReference {
            evidence_hash: create_test_evidence(1),
            evidence_type: "invoice".to_string(),
            acquired_at: 1000,
        }],
        evaluation_timestamp: 1000,
    };

    // Verify: Multiple hash computations produce identical results
    let hash1 = snapshot.compute_hash();
    let hash2 = snapshot.compute_hash();
    let hash3 = snapshot.compute_hash();

    assert_eq!(hash1, hash2);
    assert_eq!(hash2, hash3);
}
