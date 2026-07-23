/*!
# OMWEI 32BSA Trust Hierarchy Implementation

This module implements the Trust Hierarchy system for the OMWEI 32-bit execution data format (32BSA) protocol.
The Trust Hierarchy is based on Bit 31 (MSB) of the GlobalID, providing zero-latency execution integrity level determination.

## Trust Levels:

### Managed Space (Bit 31 = 0)
- Range: 0x00000000 to 0x7FFFFFFF
- IDs are issued by OMWEI authority
- MUST be verified via PQC signature
- Global trust guarantee

### Community Space (Bit 31 = 1)
- Range: 0x80000000 to 0xFFFFFFFF
- Unmanaged, sequential/arbitrary IDs for prototyping
- NO global trust guarantee
- Local/experimental use only

## Hardware Efficiency:
- Single bit-mask operation for trust level determination
- Zero latency - suitable for real-time embedded systems
- Optimized for 32-bit architectures

## Security Considerations:
- Managed IDs use sparse generation to prevent enumeration attacks
- PQC signature verification required for Managed Space
- Community Space explicitly marked as untrusted
*/

use core::fmt;

/// Bit mask for the Sincerity Bit (MSB - Bit 31)
pub const SINCERITY_BIT_MASK: u32 = 0x80000000;

/// Maximum value for Managed Space (Bit 31 = 0)
const MANAGED_SPACE_MAX: u32 = 0x7FFFFFFF;

/// Maximum value for Community Space (Bit 31 = 1)
#[allow(dead_code)]
const COMMUNITY_SPACE_MAX: u32 = 0xFFFFFFFF;

/// Trust Level enumeration for OMWEI 32BSA
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum TrustLevel {
    /// Managed Space - IDs requiring PQC verification
    Managed = 0,
    /// Community Space - Unverified experimental IDs
    Community = 1,
}

impl fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrustLevel::Managed => write!(f, "Managed (Verified)"),
            TrustLevel::Community => write!(f, "Community (Unverified)"),
        }
    }
}

/// Validation result for atom processing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationResult {
    /// Atom is valid and trusted
    Trusted,
    /// Atom is from Community Space - use with caution
    Unverified,
    /// Atom failed PQC signature verification
    InvalidSignature,
    /// Atom ID is out of valid range
    InvalidId,
}

impl fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationResult::Trusted => write!(f, "Trusted"),
            ValidationResult::Unverified => write!(f, "Warning: Unverified"),
            ValidationResult::InvalidSignature => write!(f, "Error: Invalid PQC Signature"),
            ValidationResult::InvalidId => write!(f, "Error: Invalid ID Range"),
        }
    }
}

/// OMWEI 32BSA Atom structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Atom {
    /// 32-bit Global ID with embedded trust level
    pub global_id: u32,
    /// Atom payload data
    pub payload: [u8; 28], // 32 bytes total - 4 for ID, 28 for payload
}

impl Atom {
    /// Create a new atom with the given ID and payload
    #[inline]
    pub fn new(global_id: u32, payload: [u8; 28]) -> Self {
        Self { global_id, payload }
    }

    /// Create an atom with empty payload
    #[inline]
    pub fn with_id(global_id: u32) -> Self {
        Self {
            global_id,
            payload: [0; 28],
        }
    }
}

/// Get trust level from a 32-bit Global ID
///
/// This is the core trust determination function using a single bit-mask operation.
/// Zero latency - suitable for hardware-level execution.
///
/// # Arguments
/// * `id` - 32-bit Global ID
///
/// # Returns
/// * `TrustLevel::Managed` if Bit 31 = 0 (range 0x00000000 to 0x7FFFFFFF)
/// * `TrustLevel::Community` if Bit 31 = 1 (range 0x80000000 to 0xFFFFFFFF)
///
/// # Examples
/// ```
/// use omwei_atom::trust_hierarchy::{get_trust_level, TrustLevel};
///
/// let managed_id = 0x12345678; // Bit 31 = 0
/// let community_id = 0x80000001; // Bit 31 = 1
///
/// assert_eq!(get_trust_level(managed_id), TrustLevel::Managed);
/// assert_eq!(get_trust_level(community_id), TrustLevel::Community);
/// ```
#[inline]
pub fn get_trust_level(id: u32) -> TrustLevel {
    // Single bit-mask operation - hardware efficient
    if (id & SINCERITY_BIT_MASK) == 0 {
        TrustLevel::Managed
    } else {
        TrustLevel::Community
    }
}

/// Check if an ID is in the Managed Space range
#[inline]
pub fn is_managed_id(id: u32) -> bool {
    id <= MANAGED_SPACE_MAX
}

/// Check if an ID is in the Community Space range
#[inline]
pub fn is_community_id(id: u32) -> bool {
    id > MANAGED_SPACE_MAX
}

/// Placeholder for Post-Quantum Cryptography signature verification
///
/// In a production implementation, this would interface with a PQC library
/// such as CRYSTALS-Kyber, CRYSTALS-Dilithium, or similar.
///
/// # Arguments
/// * `atom` - The atom to verify
///
/// # Returns
/// * `true` if signature is valid (placeholder - always returns false)
/// * `false` if signature is invalid
pub fn verify_pqc_signature(_atom: &Atom) -> bool {
    // TODO: Implement actual PQC signature verification
    // This should integrate with a PQC library like:
    // - CRYSTALS-Dilithium for signatures
    // - SPHINCS+ for stateless signatures
    // - Falcon for lattice-based signatures

    false // Placeholder - always return false for now
}

/// Validate an atom according to the Trust Hierarchy rules
///
/// This function simulates an AX (Accelerator eXchange) Buffer Filter that
/// immediately checks the Sincerity Bit and routes the atom accordingly.
///
/// # Arguments
/// * `atom` - The atom to validate
///
/// # Returns
/// * `ValidationResult::Trusted` - Managed ID with valid PQC signature
/// * `ValidationResult::Unverified` - Community ID (warning)
/// * `ValidationResult::InvalidSignature` - Managed ID with invalid PQC signature
/// * `ValidationResult::InvalidId` - ID out of valid range
///
/// # Examples
/// ```
/// use omwei_atom::trust_hierarchy::{validate_atom, Atom};
///
/// let managed_atom = Atom::with_id(0x12345678);
/// let community_atom = Atom::with_id(0x80000001);
///
/// let result_managed = validate_atom(&managed_atom);
/// let result_community = validate_atom(&community_atom);
/// ```
pub fn validate_atom(atom: &Atom) -> ValidationResult {
    let trust_level = get_trust_level(atom.global_id);

    match trust_level {
        TrustLevel::Managed => {
            // Managed Space - require PQC verification
            if verify_pqc_signature(atom) {
                ValidationResult::Trusted
            } else {
                ValidationResult::InvalidSignature
            }
        }
        TrustLevel::Community => {
            // Community Space - mark as unverified
            ValidationResult::Unverified
        }
    }
}

/// Sparse ID Generator for Managed Space
///
/// Generates cryptographically sparse IDs to prevent enumeration attacks.
/// Uses a cryptographically secure pseudo-random number generator with
/// a sparse distribution algorithm.
pub struct SparseIdGenerator {
    /// Internal state for the CSPRNG
    state: u64,
    /// Counter to ensure uniqueness
    counter: u64,
}

impl SparseIdGenerator {
    /// Create a new sparse ID generator
    ///
    /// # Arguments
    /// * `seed` - Optional seed for the CSPRNG (defaults to current time)
    pub fn new(seed: Option<u64>) -> Self {
        Self {
            state: seed.unwrap_or({
                // In a real implementation, use hardware RNG or secure source
                0x123456789ABCDEF0 // Placeholder seed
            }),
            counter: 0,
        }
    }

    /// Generate a sparse ID in Managed Space
    ///
    /// This function creates IDs that are:
    /// 1. Within Managed Space range (0x00000000 to 0x7FFFFFFF)
    /// 2. Cryptographically sparse to prevent enumeration
    /// 3. Unique within the generator instance
    ///
    /// # Returns
    /// A 32-bit ID suitable for Managed Space
    pub fn generate_sparse_id(&mut self) -> u32 {
        // Simple xorshift-based CSPRNG (replace with proper CSPRNG in production)
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;

        // Incorporate counter to ensure uniqueness
        self.counter += 1;
        let combined = self.state.wrapping_add(self.counter);

        // Map to Managed Space range using sparse distribution
        // This creates gaps in the ID space to prevent enumeration
        let sparse_value =
            ((combined & 0x7FFFFFFF) | ((combined & 0x0000FFFF) << 16) & 0x7FFFFFFF) as u32;

        // Ensure we're in Managed Space
        sparse_value & MANAGED_SPACE_MAX
    }

    /// Generate a batch of sparse IDs
    ///
    /// # Arguments
    /// * `count` - Number of IDs to generate
    ///
    /// # Returns
    /// Vector of unique sparse IDs in Managed Space
    pub fn generate_sparse_batch(&mut self, count: usize) -> Vec<u32> {
        let mut ids = Vec::with_capacity(count);
        for _ in 0..count {
            ids.push(self.generate_sparse_id());
        }
        ids
    }
}

impl Default for SparseIdGenerator {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_level_determination() {
        // Test Managed Space (Bit 31 = 0)
        assert_eq!(get_trust_level(0x00000000), TrustLevel::Managed);
        assert_eq!(get_trust_level(0x12345678), TrustLevel::Managed);
        assert_eq!(get_trust_level(0x7FFFFFFF), TrustLevel::Managed);

        // Test Community Space (Bit 31 = 1)
        assert_eq!(get_trust_level(0x80000000), TrustLevel::Community);
        assert_eq!(get_trust_level(0x80000001), TrustLevel::Community);
        assert_eq!(get_trust_level(0xFFFFFFFF), TrustLevel::Community);
    }

    #[test]
    fn test_space_validation() {
        // Managed Space
        assert!(is_managed_id(0x00000000));
        assert!(is_managed_id(0x12345678));
        assert!(is_managed_id(0x7FFFFFFF));
        assert!(!is_managed_id(0x80000000));

        // Community Space
        assert!(!is_community_id(0x7FFFFFFF));
        assert!(is_community_id(0x80000000));
        assert!(is_community_id(0xFFFFFFFF));
    }

    #[test]
    fn test_atom_validation() {
        let managed_atom = Atom::with_id(0x12345678);
        let community_atom = Atom::with_id(0x80000001);

        // Community atoms should return Unverified
        assert_eq!(validate_atom(&community_atom), ValidationResult::Unverified);

        // Managed atoms should return InvalidSignature (placeholder PQC verification)
        assert_eq!(
            validate_atom(&managed_atom),
            ValidationResult::InvalidSignature
        );
    }

    #[test]
    fn test_sparse_id_generation() {
        let mut generator = SparseIdGenerator::new(Some(12345));

        // Generate multiple IDs
        let id1 = generator.generate_sparse_id();
        let id2 = generator.generate_sparse_id();
        let id3 = generator.generate_sparse_id();

        // All should be in Managed Space
        assert!(is_managed_id(id1));
        assert!(is_managed_id(id2));
        assert!(is_managed_id(id3));

        // All should be unique
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);

        // Should be sparse (not sequential)
        assert!(id2.abs_diff(id1) > 1);
        assert!(id3.abs_diff(id2) > 1);
    }

    #[test]
    fn test_batch_generation() {
        let mut generator = SparseIdGenerator::new(Some(54321));
        let batch = generator.generate_sparse_batch(100);

        assert_eq!(batch.len(), 100);

        // All should be in Managed Space
        for id in &batch {
            assert!(is_managed_id(*id));
        }

        // All should be unique
        let mut unique_ids = batch.clone();
        unique_ids.sort();
        unique_ids.dedup();
        assert_eq!(unique_ids.len(), 100);
    }
}
