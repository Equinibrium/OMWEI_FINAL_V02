# sams-execution-boundary-poc

Reference implementation of an execution boundary gate that evaluates state-transition claims against typed predicates and produces a deterministic, hash-verifiable accountability record.

## Overview

This crate provides the core execution boundary logic for the SAMS (State-Aware Multi-Agent System) architecture, implementing the Observation Closure Model for deterministic execution integrity. It ensures that autonomous agent decisions are verifiable, reproducible, and causally accountable through formal observation closures.

For the formal specification and proof of the Observation Closure Theorem, see [SPECIFICATION.md](SPECIFICATION.md).

## Example Usage

```rust
use sams_execution_boundary_poc::{ExecutionBoundaryGate, ExecutionIntegrityClaim, FinancialLimitPredicate};

// Create a predicate
let predicate = FinancialLimitPredicate::new(1_000_000);

// Create a boundary gate
let gate = ExecutionBoundaryGate::new(predicate);

// Evaluate a state transition claim
let claim = ExecutionIntegrityClaim::new(/* ... */);
let result = gate.evaluate_transition(&claim)?;
```

## License

MIT OR Apache-2.0
