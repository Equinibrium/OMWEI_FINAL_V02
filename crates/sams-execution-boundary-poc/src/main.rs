//! SAMS Execution Integrity Boundary PoC - Main Demo
//! 
//! Demonstrates three scenarios:
//! A. Valid transition (success)
//! B. Constraint violation (rejection)
//! C. Extended path routing (deferred verification)

use sams_execution_boundary_poc::{
    ExecutionIntegrityClaim, ExecutionBoundaryGate, FinancialLimitPredicate,
    RequestedTransition, TransitionConstraints, SystemState, TransitionPredicate,
    create_test_atom, create_test_evidence,
    create_boundary_descriptor, create_evaluation_snapshot,
    hex,
};

fn print_separator() {
    println!("{}", "=".repeat(80));
}

fn print_scenario_header(scenario: &str, description: &str) {
    print_separator();
    println!("SCENARIO: {}", scenario);
    println!("{}", description);
    print_separator();
}

fn print_claim_details(claim: &ExecutionIntegrityClaim, snapshot_hash: &[u8]) {
    println!("\n--- Claim Details ---");
    println!("Actor:                {}", claim.actor_ref);
    println!("Verification Path Bit: {} ({})",
        claim.verification_path_bit,
        if claim.verification_path_bit { "Extended Path" } else { "Inline/Direct" }
    );
    println!("Evidence References:   {}", claim.evidence_refs.len());
    println!("Max Amount Constraint:  {:?}", claim.constraints.max_amount);
    println!("Transition:           {:?}", claim.requested_transition);
    println!("Snapshot Hash:         {}", hex(snapshot_hash));
}

fn print_state_snapshot(state: &SystemState, label: &str) {
    println!("\n--- {} State Snapshot ---", label);
    println!("Account Balances:");
    if state.account_balances.is_empty() {
        println!("  (empty)");
    } else {
        for (account, balance) in &state.account_balances {
            println!("  {}: ${}", account, balance);
        }
    }
    println!("State Hash:           {}", hex(&state.compute_hash()[..8]));
}

fn main() {
    println!("\n");
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                                                                              ║");
    println!("║        SAMS Execution Integrity Boundary PoC                                 ║");
    println!("║        State Transition Integrity Boundary Demonstration                    ║");
    println!("║                                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();

    // ============================================================================
    // SCENARIO A: Valid Transition (Success)
    // ============================================================================
    print_scenario_header(
        "SCENARIO A",
        "Valid Transition - Agent transfers $10,000 with valid evidence (Inline Path)"
    );

    let mut state_a = SystemState::new();
    state_a.account_balances.insert("recipient@example.com".to_string(), 100_000);
    
    print_state_snapshot(&state_a, "BEFORE");

    // Create claim with verification_path_bit = 0 (inline path)
    let claim_a = ExecutionIntegrityClaim::new(
        RequestedTransition::TransferFunds {
            amount: 10_000,
            recipient: "recipient@example.com".to_string(),
        },
        "agent-liquidity-manager".to_string(),
        vec![create_test_evidence(1)], // Valid invoice evidence
        TransitionConstraints {
            max_amount: Some(50_000),
            required_evidence_count: 1,
            ..Default::default()
        },
        create_test_atom(0x00000001), // Bit 31 = 0 → Inline path
    );

    let predicate_a = FinancialLimitPredicate::new(50_000, true);
    let boundary_a = create_boundary_descriptor();
    let snapshot_a = create_evaluation_snapshot(
        predicate_a.descriptor(),
        boundary_a.clone(),
        state_a.compute_hash(),
        vec![create_test_evidence(1)],
    );

    print_claim_details(&claim_a, &snapshot_a.compute_hash());

    let gate_a = ExecutionBoundaryGate::new(
        Box::new(predicate_a),
        boundary_a,
    );

    println!("\n--- Processing Transition ---");
    match gate_a.process_transition(claim_a, snapshot_a, &mut state_a) {
        Ok(record) => {
            println!("✅ TRANSITION ALLOWED");
            println!("{}", record);
        }
        Err(e) => {
            println!("❌ TRANSITION REJECTED: {}", e);
        }
    }

    print_state_snapshot(&state_a, "AFTER");

    // ============================================================================
    // SCENARIO B: Constraint Violation (Rejection)
    // ============================================================================
    println!("\n\n");
    print_scenario_header(
        "SCENARIO B",
        "Constraint Violation - Agent attempts $100,000 transfer (exceeds limit)"
    );

    let mut state_b = SystemState::new();
    state_b.account_balances.insert("recipient@example.com".to_string(), 100_000);
    
    print_state_snapshot(&state_b, "BEFORE");

    // Create claim with amount exceeding limit
    let claim_b = ExecutionIntegrityClaim::new(
        RequestedTransition::TransferFunds {
            amount: 100_000, // Exceeds $50,000 limit
            recipient: "recipient@example.com".to_string(),
        },
        "agent-liquidity-manager".to_string(),
        vec![create_test_evidence(2)],
        TransitionConstraints {
            max_amount: Some(50_000),
            required_evidence_count: 1,
            ..Default::default()
        },
        create_test_atom(0x00000002), // Bit 31 = 0 → Inline path
    );

    let predicate_b = FinancialLimitPredicate::new(50_000, true);
    let boundary_b = create_boundary_descriptor();
    let snapshot_b = create_evaluation_snapshot(
        predicate_b.descriptor(),
        boundary_b.clone(),
        state_b.compute_hash(),
        vec![create_test_evidence(2)],
    );

    print_claim_details(&claim_b, &snapshot_b.compute_hash());

    let gate_b = ExecutionBoundaryGate::new(
        Box::new(predicate_b),
        boundary_b,
    );

    println!("\n--- Processing Transition ---");
    match gate_b.process_transition(claim_b, snapshot_b, &mut state_b) {
        Ok(record) => {
            println!("✅ TRANSITION ALLOWED");
            println!("{}", record);
        }
        Err(e) => {
            println!("❌ TRANSITION REJECTED: {}", e);
        }
    }

    print_state_snapshot(&state_b, "AFTER");
    println!("Note: State remains UNCHANGED due to predicate failure");

    // ============================================================================
    // SCENARIO C: Extended Path Routing
    // ============================================================================
    println!("\n\n");
    print_scenario_header(
        "SCENARIO C",
        "Extended Path Routing - Claim requires external verification (Bit 31 = 1)"
    );

    let mut state_c = SystemState::new();
    state_c.account_balances.insert("recipient@example.com".to_string(), 100_000);
    
    print_state_snapshot(&state_c, "BEFORE");

    // Create claim with verification_path_bit = 1 (extended path)
    let claim_c = ExecutionIntegrityClaim::new(
        RequestedTransition::TransferFunds {
            amount: 25_000,
            recipient: "recipient@example.com".to_string(),
        },
        "agent-liquidity-manager".to_string(),
        vec![create_test_evidence(3)],
        TransitionConstraints {
            max_amount: Some(50_000),
            required_evidence_count: 1,
            ..Default::default()
        },
        create_test_atom(0x80000001), // Bit 31 = 1 → Extended path
    );

    let predicate_c = FinancialLimitPredicate::new(50_000, true);
    let boundary_c = create_boundary_descriptor();
    let snapshot_c = create_evaluation_snapshot(
        predicate_c.descriptor(),
        boundary_c.clone(),
        state_c.compute_hash(),
        vec![create_test_evidence(3)],
    );

    print_claim_details(&claim_c, &snapshot_c.compute_hash());

    let gate_c = ExecutionBoundaryGate::new(
        Box::new(predicate_c),
        boundary_c,
    );

    println!("\n--- Processing Transition ---");
    match gate_c.process_transition(claim_c, snapshot_c, &mut state_c) {
        Ok(record) => {
            println!("✅ TRANSITION ALLOWED");
            println!("{}", record);
        }
        Err(e) => {
            println!("❌ TRANSITION REJECTED: {}", e);
            println!("Note: Claim routed to extended verification path");
        }
    }

    print_state_snapshot(&state_c, "AFTER");
    println!("Note: State remains UNCHANGED - deferred to extended verification");

    // ============================================================================
    // SUMMARY TABLE
    // ============================================================================
    println!("\n\n");
    print_separator();
    println!("EXECUTION SUMMARY");
    print_separator();
    println!();
    println!("┌─────────────┬──────────────────┬─────────────────┬────────────────┐");
    println!("│ Scenario    │ Path Type        │ Amount          │ Result         │");
    println!("├─────────────┼──────────────────┼─────────────────┼────────────────┤");
    println!("│ Scenario A  │ Inline (Bit 31=0) │ $10,000         │ ✅ ALLOWED     │");
    println!("│ Scenario B  │ Inline (Bit 31=0) │ $100,000        │ ❌ REJECTED    │");
    println!("│ Scenario C  │ Extended (Bit=1)  │ $25,000         │ ❌ DEFERRED    │");
    println!("└─────────────┴──────────────────┴─────────────────┴────────────────┘");
    println!();
    println!("Key Observations:");
    println!("  • Scenario A: Valid amount + evidence + inline path = SUCCESS");
    println!("  • Scenario B: Amount exceeds limit = REJECTION (state unchanged)");
    println!("  • Scenario C: Extended path bit set = DEFERRED to external verification");
    println!();
    print_separator();
    println!();
}
