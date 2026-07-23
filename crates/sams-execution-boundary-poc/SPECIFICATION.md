# SAMS Execution Boundary - Observation Closure Model Specification

## Overview

This specification defines the **Observation Closure Model** as the foundational abstraction for deterministic execution integrity in the SAMS (State-Aware Multi-Agent System) architecture. The model ensures that autonomous agent decisions are verifiable, reproducible, and causally accountable through formal observation closures.

## Core Principle: Observation Closure

**Definition**: An execution is deterministic if and only if every observable input influencing the evaluation is explicitly captured within the `EvaluationSnapshot`.

**Formal Property**:
```
same_claim + same_snapshot + same_predicate_implementation = same_decision
```

**Observation Closure Theorem**: For any pure predicate P, if `hash(S1) == hash(S2)`, then `P(S1) == P(S2)`. Conversely, if `P(S1) != P(S2)`, then `hash(S1) != hash(S2)`.

---

# Stage 1 Specification

## 1. Core Types

### 1.1 PredicateDescriptor

Identifies predicate implementation and version.

```rust
pub struct PredicateDescriptor {
    pub predicate_id: String,           // e.g., "financial_limit_v1"
    pub version: u32,                   // Semantic version number
    pub implementation_hash: [u8; 32],  // SHA256 of predicate bytecode
}
```

**Properties**:
- `compute_hash() -> [u8; 32]`: Canonical hash of descriptor
- Hash includes: predicate_id, version, implementation_hash

### 1.2 BoundaryDescriptor

Identifies boundary implementation and routing semantics.

```rust
pub struct BoundaryDescriptor {
    pub semantic_version: String,        // e.g., "1.0.0"
    pub implementation_hash: [u8; 32],  // SHA256 of boundary code
    pub routing_semantics_version: u32, // For Bit 31 interpretation changes
}
```

**Properties**:
- `compute_hash() -> [u8; 32]`: Canonical hash of descriptor
- Hash includes: semantic_version, implementation_hash, routing_semantics_version

### 1.3 ReferencedState

What system state was observed during evaluation.

```rust
pub struct ReferencedState {
    pub state_id: String,       // e.g., "account_ledger", "robot_position"
    pub state_hash: [u8; 32],   // Content address of state snapshot
}
```

### 1.4 EvidenceReference

What external evidence was available during evaluation.

```rust
pub struct EvidenceReference {
    pub evidence_hash: [u8; 32],    // Content address of evidence
    pub evidence_type: String,      // e.g., "market_snapshot", "compliance_check"
    pub acquired_at: u64,           // Timestamp when evidence was acquired
}
```

### 1.5 EvaluationSnapshot

The complete deterministic closure containing all observable inputs.

```rust
pub struct EvaluationSnapshot {
    pub predicate: PredicateDescriptor,
    pub boundary: BoundaryDescriptor,
    pub referenced_states: Vec<ReferencedState>,
    pub evidence_references: Vec<EvidenceReference>,
    pub evaluation_timestamp: u64,  // Metadata for audit trail (not part of determinism)
}
```

**Properties**:
- `compute_hash() -> [u8; 32]`: Canonical hash of observation closure
- `is_observation_equivalent(&self, other: &EvaluationSnapshot) -> bool`: Check if two snapshots represent identical observations
- **Critical**: `evaluation_timestamp` does NOT affect the hash (metadata only)

**Canonical Serialization**:
The hash computation uses deterministic byte encoding:
1. Hash of predicate descriptor
2. Hash of boundary descriptor
3. For each referenced state: state_id bytes + state_hash
4. For each evidence reference: evidence_hash + evidence_type bytes + acquired_at (little-endian)
5. All uses SHA256 with consistent byte ordering (little-endian for integers)

## 2. Execution Integrity Claim

### 2.1 ExecutionIntegrityClaim

The autonomous decision/intent submitted for evaluation.

```rust
pub struct ExecutionIntegrityClaim {
    pub requested_transition: RequestedTransition,
    pub actor_ref: String,
    pub evidence_refs: Vec<[u8; 32]>,
    pub constraints: TransitionConstraints,
    pub verification_path_bit: bool,  // Extracted from Atom Bit 31
}
```

**Properties**:
- `compute_hash() -> [u8; 32]`: Canonical hash of claim
- `verification_path_bit`: If true, routes to extended verification path

### 2.2 RequestedTransition

The proposed state transition.

```rust
pub enum RequestedTransition {
    TransferFunds { amount: u64, recipient: String },
    UpdateConfiguration { key: String, value: String },
    ExecuteCommand { command: String, args: Vec<String> },
}
```

### 2.3 TransitionConstraints

Constraints applied to the transition.

```rust
pub struct TransitionConstraints {
    pub max_amount: Option<u64>,
    pub time_window_start: Option<u64>,
    pub time_window_end: Option<u64>,
    pub required_evidence_count: usize,
}
```

## 3. Predicate System

### 3.1 TransitionPredicate Trait

Pure function over observation closure.

```rust
pub trait TransitionPredicate: Send + Sync {
    fn descriptor(&self) -> PredicateDescriptor;
    fn evaluate(
        &self,
        claim: &ExecutionIntegrityClaim,
        snapshot: &EvaluationSnapshot,
        current_state: &SystemState,
    ) -> Result<PredicateResult, PredicateError>;
}
```

**Invariants**:
- Must be side-effect free (no I/O, no system clock calls)
- Reads only from snapshot and state
- Identical snapshots MUST produce identical results

### 3.2 PredicateResult

Possible outcomes of predicate evaluation.

```rust
pub enum PredicateResult {
    Approved,
    Rejected,
    IncompleteEvidence { missing: Vec<[u8; 32]> },
}
```

### 3.3 PredicateError

Errors that can occur during predicate evaluation.

```rust
pub enum PredicateError {
    DescriptorMismatch,
    UnsupportedTransitionType(String),
    StateNotFound(String),
    EvidenceNotFound(String),
    InternalError(String),
}
```

## 4. Execution Boundary Gate

### 4.1 ExecutionBoundaryGate

Enforces the observation closure model and routes transitions.

```rust
pub struct ExecutionBoundaryGate {
    predicate: Box<dyn TransitionPredicate>,
    boundary_descriptor: BoundaryDescriptor,
}
```

**Methods**:
- `process_transition(claim, snapshot, state) -> Result<CausalAccountabilityRecord, BoundaryError>`
- `verify_snapshot_consistency(snapshot) -> Result<(), BoundaryError>`

**Process Flow**:
1. Verify snapshot consistency (boundary descriptor match)
2. Route based on verification_path_bit (extended path if true)
3. Evaluate predicate (pure function)
4. Conditional state mutation (only if Approved)
5. Generate accountability record

### 4.2 BoundaryError

Errors that can occur at the boundary.

```rust
pub enum BoundaryError {
    ExtendedVerificationRequired,
    PredicateRejected,
    IncompleteEvidence(String),
    PredicateError(String),
    InvalidTransition(String),
    DeterminismViolation,  // Snapshot hash mismatch or non-deterministic behavior
}
```

## 5. Causal Accountability Record

### 5.1 CausalAccountabilityRecord

Immutable audit entry containing the observation closure and decision.

```rust
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
```

**Properties**:
- Sufficient for replay and verification
- Contains complete observation closure via snapshot_hash
- Records state transition via before/after hashes

## 6. Determinism Verification

### 6.1 DeterminismVerifier

Utility for verifying determinism properties.

```rust
pub struct DeterminismVerifier;
```

**Methods**:
- `are_observation_equivalent(snapshot1, snapshot2) -> bool`: Check if snapshots have identical hashes
- `verify_determinism(predicate, claim, snapshot, state) -> Result<PredicateResult, PredicateError>`: Run predicate twice to verify identical results
- `are_records_deterministic_equivalent(record1, record2) -> bool`: Check if records represent identical execution

## 7. System State

### 7.1 SystemState

The mutable system state that transitions occur on.

```rust
pub struct SystemState {
    pub account_balances: HashMap<String, u64>,
    pub configuration: HashMap<String, String>,
    pub last_transition_id: Option<String>,
}
```

**Properties**:
- `compute_hash() -> [u8; 32]`: Canonical hash of state
- Hash includes: all account balances, configuration entries, last transition ID

---

# Formal Observation Closure Theorem

## Theorem Statement

**For any pure predicate P, if hash(S1) == hash(S2), then P(S1) == P(S2).**

**Contrapositive**: If P(S1) != P(S2), then hash(S1) != hash(S2).

## Proof

### Lemma 1: Canonical Serialization is Deterministic

The `EvaluationSnapshot::compute_hash()` function uses a deterministic byte encoding:
- All integers use little-endian encoding
- String bytes are used directly
- Collection order is preserved
- SHA256 is a deterministic cryptographic hash function

Therefore, for any snapshot S, `compute_hash(S)` is architecture-independent and reproducible.

### Lemma 2: Pure Predicates are Deterministic

By definition of the `TransitionPredicate` trait:
- Predicates have no side effects
- Predicates read only from their inputs (claim, snapshot, state)
- Predicates do not perform I/O or read system clocks

Therefore, for any pure predicate P and identical inputs (claim, snapshot, state), `P(claim, snapshot, state)` always produces the same result.

### Theorem Proof

Given:
- Two snapshots S1 and S2 such that `hash(S1) == hash(S2)`
- A pure predicate P
- Identical claim and state

From `hash(S1) == hash(S2)` and Lemma 1, we know that S1 and S2 have identical byte representations, meaning:
- `S1.predicate == S2.predicate`
- `S1.boundary == S2.boundary`
- `S1.referenced_states == S2.referenced_states`
- `S1.evidence_references == S2.evidence_references`

Since the predicate reads only from the snapshot and the snapshots are identical, the predicate receives identical inputs.

From Lemma 2, pure predicates are deterministic, so:
```
P(claim, S1, state) == P(claim, S2, state)
```

QED.

## Determinism Violation

A determinism violation occurs when:
1. Two snapshots with identical hashes produce different predicate results
2. This indicates either:
   - The predicate is not pure (has side effects or non-deterministic behavior)
   - The hash computation is non-deterministic
   - The snapshot construction is inconsistent

In the Observation Closure Model, determinism violations are detected via:
- `DeterminismVerifier::verify_determinism()` - runs predicate twice with identical inputs
- `BoundaryError::DeterminismViolation` - raised when snapshot inconsistencies are detected

---

# Stage 2 Roadmap

## Distributed Snapshots

**Goal**: Enable observation closure across distributed system boundaries.

**Planned Features**:
- Cross-node snapshot synchronization
- Merkle tree-based state references
- Distributed evidence aggregation
- Consensus on snapshot hashes

## Ghost Node Integration

**Goal**: Integrate with Ghost Node architecture for enhanced verification.

**Planned Features**:
- Ghost node as extended verification path
- Snapshot attestation by ghost nodes
- Cryptographic proof of observation closure
- Multi-party predicate evaluation

## Advanced Predicate Types

**Goal**: Support more complex predicate logic while maintaining purity.

**Planned Features**:
- Composable predicates (AND, OR, NOT)
- Temporal predicates (time-window constraints)
- State-machine predicates
- Policy composition language

## Performance Optimizations

**Goal**: Scale observation closure model to high-throughput systems.

**Planned Features**:
- Incremental hash computation
- Snapshot caching and reuse
- Parallel predicate evaluation
- Bloom filters for evidence lookup

---

# Implementation Notes

## Hash Computation Details

All hash computations use SHA256 with the following conventions:
- Integers are encoded in little-endian byte order
- Strings are encoded as UTF-8 bytes
- Collections are hashed in iteration order
- Empty collections contribute no bytes to the hash

## Thread Safety

The following types are thread-safe:
- `EvaluationSnapshot` (immutable after creation)
- `PredicateDescriptor` (immutable)
- `BoundaryDescriptor` (immutable)
- `TransitionPredicate` implementations (must be `Send + Sync`)

## Error Handling

- Predicate errors are typed via `PredicateError`
- Boundary errors are typed via `BoundaryError`
- All errors are recoverable and informative
- Determinism violations are explicitly detected and reported

## Testing Strategy

The test suite verifies:
1. **Observation Closure Theorem**: Identical snapshots produce identical results
2. **Hash Determinism**: Multiple hash computations produce identical results
3. **Canonical Serialization**: Hash is architecture-independent
4. **Determinism Verification**: Non-deterministic behavior is detected
5. **State Mutation**: State changes only on approved transitions
6. **Path Routing**: Verification path bit correctly routes transitions

---

# Version History

- **v0.1.0** (Stage 1): Initial Observation Closure Model
  - Core types and descriptors
  - Predicate system with pure function trait
  - Execution boundary gate
  - Determinism verifier
  - Causal accountability records
  - Integration tests for Observation Closure Theorem
