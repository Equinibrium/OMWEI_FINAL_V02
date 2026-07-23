/*!
# OMWEI 32BSA Trust Hierarchy Demonstration

This example demonstrates the Trust Hierarchy system for OMWEI 32-bit execution data format.
It shows how to:

1. Determine execution integrity levels from Global IDs
2. Validate atoms according to execution integrity rules
3. Generate sparse IDs for Managed Space
4. Simulate AX Buffer Filter operations

## Key Concepts:

- **Managed Space (Bit 31 = 0):** 0x00000000 to 0x7FFFFFFF - IDs requiring PQC verification
- **Community Space (Bit 31 = 1):** 0x80000000 to 0xFFFFFFFF - Unverified experimental IDs
- **Zero Latency:** Single bit-mask operation for execution integrity determination
*/

use omwei_atom::trust_hierarchy::{
    get_trust_level, validate_atom, Atom, SparseIdGenerator, ValidationResult, SINCERITY_BIT_MASK,
};

fn main() {
    println!("=== OMWEI 32BSA Trust Hierarchy Demonstration ===\n");

    // 1. Trust Level Determination
    println!("1. Trust Level Determination");
    println!("   Sincerity Bit Mask: 0x{:08X}\n", SINCERITY_BIT_MASK);

    let test_ids = [
        0x00000000, // Managed Space - start
        0x12345678, // Managed Space - middle
        0x7FFFFFFF, // Managed Space - end
        0x80000000, // Community Space - start
        0x80000001, // Community Space - middle
        0xFFFFFFFF, // Community Space - end
    ];

    for &id in &test_ids {
        let trust_level = get_trust_level(id);
        let binary_str = format!("{:032b}", id);
        let _msb = if (id & 0x80000000) == 0 { "0" } else { "1" };

        println!(
            "   ID: 0x{:08X} | Binary: {}...{} | Trust: {}",
            id,
            &binary_str[..8],
            &binary_str[24..],
            trust_level
        );
    }

    // 2. Atom Validation Simulation
    println!("\n2. AX Buffer Filter Simulation");

    let managed_atom = Atom::new(0x12345678, [0x42; 28]);
    let community_atom = Atom::new(0x80000001, [0x24; 28]);

    println!(
        "   Managed Atom (0x{:08X}): {}",
        managed_atom.global_id,
        validate_atom(&managed_atom)
    );
    println!(
        "   Community Atom (0x{:08X}): {}",
        community_atom.global_id,
        validate_atom(&community_atom)
    );

    // 3. Sparse ID Generation
    println!("\n3. Sparse ID Generation for Managed Space");

    let mut generator = SparseIdGenerator::new(Some(0xDEADBEEF));

    println!("   Generating 10 sparse IDs:");
    let sparse_ids = generator.generate_sparse_batch(10);

    for (i, &id) in sparse_ids.iter().enumerate() {
        println!(
            "   ID {}: 0x{:08X} | Trust: {}",
            i + 1,
            id,
            get_trust_level(id)
        );
    }

    // Verify all are in Managed Space and unique
    let unique_count = sparse_ids
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len();
    println!(
        "   Generated {} unique IDs (all should be in Managed Space)",
        unique_count
    );

    // 4. Performance Demonstration
    println!("\n4. Performance Demonstration");

    let iterations = 10_000_000;
    let start_time = std::time::Instant::now();

    // Simulate high-throughput trust level checking
    for i in 0..iterations {
        let id = if i % 2 == 0 { 0x12345678 } else { 0x80000001 };
        let _trust = get_trust_level(id);
    }

    let elapsed = start_time.elapsed();
    let ops_per_second = iterations as f64 / elapsed.as_secs_f64();

    println!("   Trust level determination performance:");
    println!("   - Operations: {}", iterations);
    println!("   - Time: {:?}", elapsed);
    println!("   - Speed: {:.0} ops/sec", ops_per_second);
    println!(
        "   - Latency per operation: {:.2} ns",
        elapsed.as_nanos() as f64 / iterations as f64
    );

    // 5. Security Analysis
    println!("\n5. Security Analysis");

    // Demonstrate enumeration resistance
    let mut gen1 = SparseIdGenerator::new(Some(0x11111111));
    let mut gen2 = SparseIdGenerator::new(Some(0x22222222));

    let batch1 = gen1.generate_sparse_batch(5);
    let batch2 = gen2.generate_sparse_batch(5);

    println!("   Enumeration Resistance Test:");
    println!("   Generator 1 (seed 0x11111111): {:?}", batch1);
    println!("   Generator 2 (seed 0x22222222): {:?}", batch2);

    // Check for overlaps (should be none with good seeds)
    let set1: std::collections::HashSet<_> = batch1.iter().collect();
    let set2: std::collections::HashSet<_> = batch2.iter().collect();
    let overlap: std::collections::HashSet<_> = set1.intersection(&set2).collect();

    println!(
        "   Overlap between generators: {} (should be 0)",
        overlap.len()
    );

    // 6. Real-world Simulation
    println!("\n6. Real-world Simulation");

    simulate_iot_sensor_network();

    println!("\n=== Demonstration Complete ===");
}

fn simulate_iot_sensor_network() {
    println!("   Simulating IoT Sensor Network with Mixed Trust Levels");

    // Create sensors with different trust levels
    let sensors = [
        ("Managed Sensor A", 0x12345678),
        ("Managed Sensor B", 0x23456789),
        ("Community Sensor X", 0x80000001),
        ("Community Sensor Y", 0x80000002),
    ];

    let mut trusted_readings = 0;
    let mut unverified_readings = 0;
    let mut invalid_signatures = 0;

    for (name, id) in &sensors {
        let atom = Atom::new(*id, [rand::random(); 28]);
        let result = validate_atom(&atom);

        println!("   {}: {}", name, result);

        match result {
            ValidationResult::Trusted => trusted_readings += 1,
            ValidationResult::Unverified => unverified_readings += 1,
            ValidationResult::InvalidSignature => invalid_signatures += 1,
            _ => {}
        }
    }

    println!("   Summary:");
    println!("   - Trusted readings: {}", trusted_readings);
    println!("   - Unverified readings: {}", unverified_readings);
    println!("   - Invalid signatures: {}", invalid_signatures);
    println!(
        "   - Total readings: {}",
        trusted_readings + unverified_readings + invalid_signatures
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_hierarchy_consistency() {
        // Test that trust level determination is consistent
        for i in 0..1000 {
            let id = rand::random::<u32>();
            let trust1 = get_trust_level(id);
            let trust2 = get_trust_level(id);
            assert_eq!(
                trust1, trust2,
                "Trust level should be consistent for ID 0x{:08X}",
                id
            );
        }
    }

    #[test]
    fn test_managed_space_bounds() {
        // Test Managed Space boundaries
        assert_eq!(get_trust_level(0x00000000), TrustLevel::Managed);
        assert_eq!(get_trust_level(0x7FFFFFFF), TrustLevel::Managed);
        assert_eq!(get_trust_level(0x80000000), TrustLevel::Community);
        assert_eq!(get_trust_level(0xFFFFFFFF), TrustLevel::Community);
    }

    #[test]
    fn test_sparse_id_uniqueness() {
        // Test that sparse ID generator produces unique IDs
        let mut generator = SparseIdGenerator::new(Some(0x12345678));
        let batch = generator.generate_sparse_batch(1000);

        let mut unique_ids = batch.clone();
        unique_ids.sort();
        unique_ids.dedup();

        assert_eq!(
            batch.len(),
            unique_ids.len(),
            "All generated IDs should be unique"
        );
    }

    #[test]
    fn test_atom_validation_logic() {
        // Test atom validation logic
        let managed_atom = Atom::new(0x12345678, [0; 28]);
        let community_atom = Atom::new(0x80000001, [0; 28]);

        // Community atoms should always be unverified
        assert_eq!(validate_atom(&community_atom), ValidationResult::Unverified);

        // Managed atoms should fail signature verification (placeholder)
        assert_eq!(
            validate_atom(&managed_atom),
            ValidationResult::InvalidSignature
        );
    }
}
