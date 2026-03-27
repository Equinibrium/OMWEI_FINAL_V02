//! Semantic Engine module for OMWEI Equinibrium SoC
//! 
//! This module provides the semantic processing capabilities using the
//! SiFive Custom Instruction Extension (SCIE) for hardware-accelerated
//! semantic operations on 256-bit atoms.

pub mod scie;
pub mod navigator;

pub use scie::{Scie, ScieCapabilities};
pub use navigator::SemanticNavigator;

use core::fmt;

/// 256-bit semantic atom representing a unique identifier
/// 
/// This structure represents a semantic atom in the Equinibrium system.
/// Each atom is a 256-bit unique identifier that can be compared and
/// linked using hardware acceleration.
#[repr(C, align(32))] // 256-bit (32-byte) alignment for SCIE
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SemanticAtom {
    /// 256-bit atom data (4 x 64-bit words)
    pub data: [u64; 4],
}

/// Semantic table for storing atoms in memory
/// 
/// This structure represents a collection of semantic atoms stored
/// in RAM with proper alignment for SCIE operations.
#[repr(C, align(32))]
pub struct SemanticTable {
    /// Array of semantic atoms
    atoms: [SemanticAtom; Self::MAX_ATOMS],
    /// Current number of atoms in the table
    count: usize,
}

/// Semantic Triple representing subject-predicate-object relationships
/// 
/// This structure represents a semantic triple (subject-predicate-object) used
/// for graph traversal operations. Each component is a 256-bit semantic atom.
#[repr(C, align(32))] // 256-bit alignment for optimal memory port access
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SemanticTriple {
    /// Subject atom (who/what)
    pub subject: SemanticAtom,
    /// Predicate atom (relationship/type)
    pub predicate: SemanticAtom,
    /// Object atom (to/where)
    pub object: SemanticAtom,
}

/// Triple Store for storing semantic triples in memory
/// 
/// This structure manages a collection of semantic triples stored in RAM
/// with proper alignment for efficient hardware access.
#[repr(C, align(32))]
#[derive(Debug, Clone, Copy)]
pub struct TripleStore {
    /// Array of semantic triples
    triples: [SemanticTriple; Self::MAX_TRIPLES],
    /// Current number of triples in the store
    count: usize,
}

impl SemanticAtom {
    /// Create a new semantic atom from 256-bit data
    /// 
    /// # Arguments
    /// * `data` - Array of 4 x 64-bit words representing the atom
    pub const fn new(data: [u64; 4]) -> Self {
        Self { data }
    }
    
    /// Create a semantic atom from a single 64-bit seed
    /// 
    /// This function generates a 256-bit atom from a 64-bit seed
    /// using a simple hash function.
    /// 
    /// # Arguments
    /// * `seed` - 64-bit seed value
    pub fn from_seed(seed: u64) -> Self {
        let mut data = [0u64; 4];
        data[0] = seed;
        
        // Simple hash expansion to 256 bits
        for i in 1..4 {
            data[i] = data[i - 1].wrapping_mul(0x9E3779B97F4A7C15)
                .wrapping_add(0xBF58476D1CE4E5B9);
        }
        
        Self { data }
    }
    
    /// Create a semantic atom from random data
    /// 
    /// This function creates a semantic atom from random 256-bit data.
    /// 
    /// # Arguments
    /// * `rng_data` - Array of 4 random 64-bit values
    pub const fn from_random(rng_data: [u64; 4]) -> Self {
        Self { data: rng_data }
    }
    
    /// Get the first 64 bits of the atom for display purposes
    pub fn as_u64(&self) -> u64 {
        self.data[0]
    }
    
    /// Compare two semantic atoms using SCIE hardware acceleration
    /// 
    /// This function uses the SCIE_CMP_ATOM instruction to compare
    /// two 256-bit atoms in a single cycle.
    /// 
    /// # Arguments
    /// * `a` - First atom to compare
    /// * `b` - Second atom to compare
    /// 
    /// # Safety
    /// This function uses unsafe custom instructions. The caller must
    /// ensure both atoms are properly aligned and accessible.
    /// 
    /// # Returns
    /// Returns true if atoms are equal, false if they differ
    pub unsafe fn compare_scie(a: &SemanticAtom, b: &SemanticAtom) -> bool {
        // Use SCIE hardware acceleration for comparison
        Scie::compare_atoms(
            a as *const SemanticAtom as *const u8,
            b as *const SemanticAtom as *const u8,
        )
    }
    
    /// Compare two semantic atoms using software (fallback)
    /// 
    /// This function provides a software-based comparison for fallback
    /// when SCIE is not available.
    /// 
    /// # Arguments
    /// * `a` - First atom to compare
    /// * `b` - Second atom to compare
    /// 
    /// # Returns
    /// Returns true if atoms are equal, false if they differ
    pub fn compare_sw(a: &SemanticAtom, b: &SemanticAtom) -> bool {
        a.data == b.data
    }
    
    /// Compare two semantic atoms (automatic method selection)
    /// 
    /// This function automatically selects between SCIE hardware
    /// acceleration and software comparison based on availability.
    /// 
    /// # Arguments
    /// * `a` - First atom to compare
    /// * `b` - Second atom to compare
    /// 
    /// # Returns
    /// Returns true if atoms are equal, false if they differ
    pub fn compare(a: &SemanticAtom, b: &SemanticAtom) -> bool {
        // Try SCIE first, fallback to software if not available
        unsafe {
            let capabilities = Scie::get_capabilities();
            if capabilities.supports_compare {
                Self::compare_scie(a, b)
            } else {
                Self::compare_sw(a, b)
            }
        }
    }
    
    /// Get the atom as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const SemanticAtom as *const u8,
                core::mem::size_of::<SemanticAtom>(),
            )
        }
    }
    
    /// Get the atom as a mutable byte slice
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self as *mut SemanticAtom as *mut u8,
                core::mem::size_of::<SemanticAtom>(),
            )
        }
    }
    
    /// Get the atom data as a 256-bit integer representation
    pub fn as_u256(&self) -> [u64; 4] {
        self.data
    }
    
    /// Check if the atom is zero (all bits cleared)
    pub fn is_zero(&self) -> bool {
        self.data.iter().all(|&word| word == 0)
    }
    
    /// Check if the atom is valid (non-zero)
    pub fn is_valid(&self) -> bool {
        !self.is_zero()
    }
}

impl Default for SemanticAtom {
    fn default() -> Self {
        Self { data: [0; 4] }
    }
}

impl fmt::Display for SemanticAtom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Atom({:016x}{:016x}{:016x}{:016x})", 
               self.data[3], self.data[2], self.data[1], self.data[0])
    }
}

impl SemanticTable {
    /// Maximum number of atoms in the table
    pub const MAX_ATOMS: usize = 1024;
    
    /// Create a new empty semantic table
    pub const fn new() -> Self {
        Self {
            atoms: [SemanticAtom::new([0; 4]); Self::MAX_ATOMS],
            count: 0,
        }
    }
    
    /// Add an atom to the table
    /// 
    /// # Arguments
    /// * `atom` - The atom to add
    /// 
    /// # Returns
    /// Returns true if the atom was added, false if the table is full
    pub fn add_atom(&mut self, atom: SemanticAtom) -> bool {
        if self.count < Self::MAX_ATOMS {
            self.atoms[self.count] = atom;
            self.count += 1;
            true
        } else {
            false
        }
    }
    
    /// Get an atom by index
    /// 
    /// # Arguments
    /// * `index` - Index of the atom to retrieve
    /// 
    /// # Returns
    /// Returns Some(Atom) if index is valid, None otherwise
    pub fn get_atom(&self, index: usize) -> Option<&SemanticAtom> {
        if index < self.count {
            Some(&self.atoms[index])
        } else {
            None
        }
    }
    
    /// Get the number of atoms in the table
    pub fn len(&self) -> usize {
        self.count
    }
    
    /// Check if the table is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    
    /// Check if the table is full
    pub fn is_full(&self) -> bool {
        self.count >= Self::MAX_ATOMS
    }
    
    /// Clear all atoms from the table
    pub fn clear(&mut self) {
        self.count = 0;
    }
    
    /// Search for an atom using SCIE hardware acceleration
    /// 
    /// This function searches for a target atom in the table using
    /// SCIE hardware acceleration for fast comparison.
    /// 
    /// # Arguments
    /// * `target` - The atom to search for
    /// 
    /// # Returns
    /// Returns Some(index) if found, None otherwise
    pub unsafe fn search_scie(&self, target: &SemanticAtom) -> Option<usize> {
        let capabilities = Scie::get_capabilities();
        if !capabilities.supports_compare {
            return self.search_sw(target);
        }
        
        for i in 0..self.count {
            if SemanticAtom::compare_scie(&self.atoms[i], target) {
                return Some(i);
            }
        }
        
        None
    }
    
    /// Search for an atom using software comparison
    /// 
    /// # Arguments
    /// * `target` - The atom to search for
    /// 
    /// # Returns
    /// Returns Some(index) if found, None otherwise
    pub fn search_sw(&self, target: &SemanticAtom) -> Option<usize> {
        for i in 0..self.count {
            if SemanticAtom::compare_sw(&self.atoms[i], target) {
                return Some(i);
            }
        }
        
        None
    }
    
    /// Search for an atom (automatic method selection)
    /// 
    /// # Arguments
    /// * `target` - The atom to search for
    /// 
    /// # Returns
    /// Returns Some(index) if found, None otherwise
    pub fn search(&self, target: &SemanticAtom) -> Option<usize> {
        unsafe {
            self.search_scie(target)
        }
    }
    
    /// Batch search for multiple atoms
    /// 
    /// This function searches for multiple target atoms efficiently.
    /// 
    /// # Arguments
    /// * `targets` - Array of target atoms to search for
    /// * `results` - Array to store search results
    /// 
    /// # Safety
    /// This function uses unsafe SCIE operations
    pub unsafe fn batch_search(&self, targets: &[SemanticAtom], results: &mut [Option<usize>]) {
        let count = targets.len().min(results.len());
        
        for i in 0..count {
            results[i] = self.search(&targets[i]);
        }
    }
}

impl Default for SemanticTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Search result for parallel operations
#[repr(C, align(32))]
#[derive(Debug, Clone, Copy)]
pub struct SearchResult {
    /// The object atom that matched the predicate
    pub object: SemanticAtom,
    /// Which hart found this result
    pub found_by: usize,
    /// Index of the matching triple in the store
    pub triple_index: usize,
}

/// Work assignment for parallel search
#[repr(C, align(32))]
#[derive(Debug, Clone, Copy)]
pub struct WorkAssignment {
    /// Starting index in the triple store
    pub start_index: usize,
    /// Ending index in the triple store
    pub end_index: usize,
    /// Target predicate to search for
    pub target_predicate: SemanticAtom,
    /// Hart ID this assignment is for
    pub hart_id: usize,
}

/// Target atom for search operations (legacy compatibility)
#[repr(C, align(32))]
#[derive(Debug, Clone, Copy)]
pub struct TargetAtom {
    /// The atom to search for
    atom: SemanticAtom,
    /// Whether this target has been found
    found: bool,
    /// Which hart found it
    found_by: Option<usize>,
}

impl TargetAtom {
    /// Create a new target atom
    /// 
    /// # Arguments
    /// * `atom` - The atom to search for
    pub const fn new(atom: SemanticAtom) -> Self {
        Self {
            atom,
            found: false,
            found_by: None,
        }
    }
    
    /// Get the atom to search for
    pub const fn atom(&self) -> &SemanticAtom {
        &self.atom
    }
    
    /// Check if the target has been found
    pub const fn is_found(&self) -> bool {
        self.found
    }
    
    /// Get which hart found the target
    pub const fn found_by(&self) -> Option<usize> {
        self.found_by
    }
    
    /// Mark the target as found by a specific hart
    /// 
    /// # Arguments
    /// * `hart_id` - The hart that found the target
    pub fn mark_found(&mut self, hart_id: usize) {
        self.found = true;
        self.found_by = Some(hart_id);
    }
    
    /// Reset the target search state
    pub fn reset(&mut self) {
        self.found = false;
        self.found_by = None;
    }
}

impl SemanticTriple {
    /// Create a new semantic triple
    /// 
    /// # Arguments
    /// * `subject` - Subject atom
    /// * `predicate` - Predicate atom
    /// * `object` - Object atom
    pub const fn new(subject: SemanticAtom, predicate: SemanticAtom, object: SemanticAtom) -> Self {
        Self { subject, predicate, object }
    }
    
    /// Get the predicate atom
    pub const fn predicate(&self) -> &SemanticAtom {
        &self.predicate
    }
    
    /// Get the object atom
    pub const fn object(&self) -> &SemanticAtom {
        &self.object
    }
    
    /// Get the subject atom
    pub const fn subject(&self) -> &SemanticAtom {
        &self.subject
    }
    
    /// Check if the predicate matches a target atom
    /// 
    /// # Arguments
    /// * `target_predicate` - The predicate to compare against
    /// 
    /// # Returns
    /// Returns true if predicates match
    pub fn predicate_matches(&self, target_predicate: &SemanticAtom) -> bool {
        SemanticAtom::compare(&self.predicate, target_predicate)
    }
    
    /// Get the triple as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const SemanticTriple as *const u8,
                core::mem::size_of::<SemanticTriple>(),
            )
        }
    }
}

impl Default for SemanticTriple {
    fn default() -> Self {
        Self {
            subject: SemanticAtom::new([0; 4]),
            predicate: SemanticAtom::new([0; 4]),
            object: SemanticAtom::new([0; 4]),
        }
    }
}

impl TripleStore {
    /// Maximum number of triples in the store
    pub const MAX_TRIPLES: usize = 1024;
    
    /// Create a new empty triple store
    pub const fn new() -> Self {
        Self {
            triples: [SemanticTriple {
                subject: SemanticAtom::new([0; 4]),
                predicate: SemanticAtom::new([0; 4]),
                object: SemanticAtom::new([0; 4]),
            }; Self::MAX_TRIPLES],
            count: 0,
        }
    }
    
    /// Add a triple to the store
    /// 
    /// # Arguments
    /// * `triple` - The triple to add
    /// 
    /// # Returns
    /// Returns true if the triple was added, false if the store is full
    pub fn add_triple(&mut self, triple: SemanticTriple) -> bool {
        if self.count < Self::MAX_TRIPLES {
            self.triples[self.count] = triple;
            self.count += 1;
            true
        } else {
            false
        }
    }
    
    /// Get a triple by index
    /// 
    /// # Arguments
    /// * `index` - Index of the triple to retrieve
    /// 
    /// # Returns
    /// Returns Some(&Triple) if index is valid, None otherwise
    pub fn get_triple(&self, index: usize) -> Option<&SemanticTriple> {
        if index < self.count {
            Some(&self.triples[index])
        } else {
            None
        }
    }
    
    /// Get the number of triples in the store
    pub fn len(&self) -> usize {
        self.count
    }
    
    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    
    /// Check if the store is full
    pub fn is_full(&self) -> bool {
        self.count >= Self::MAX_TRIPLES
    }
    
    /// Clear all triples from the store
    pub fn clear(&mut self) {
        self.count = 0;
    }
    
    /// Get a slice of triples in the specified range
    /// 
    /// # Arguments
    /// * `start` - Starting index (inclusive)
    /// * `end` - Ending index (exclusive)
    /// 
    /// # Returns
    /// Returns a slice of triples in the specified range
    pub fn get_range(&self, start: usize, end: usize) -> &[SemanticTriple] {
        if start >= self.count || end > self.count || start >= end {
            return &[];
        }
        &self.triples[start..end]
    }
    
    /// Prefetch triples to warm up L2 cache
    /// 
    /// # Arguments
    /// * `start` - Starting index to prefetch
    /// * `count` - Number of triples to prefetch
    pub fn prefetch_range(&self, start: usize, count: usize) {
        let end = core::cmp::min(start + count, self.count);
        for i in start..end {
            // Dummy read to warm up cache
            let _triple = &self.triples[i];
            
            // Prefetch hint for the hardware prefetcher
            unsafe {
                // Note: riscv::asm::prefetch_read might not be available
                // This serves as a hint to the compiler and hardware
                core::ptr::read_volatile(&self.triples[i]);
            }
        }
    }
}

impl Default for TripleStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchResult {
    /// Create a new search result
    /// 
    /// # Arguments
    /// * `object` - The matching object atom
    /// * `found_by` - Which hart found this result
    /// * `triple_index` - Index of the matching triple
    pub const fn new(object: SemanticAtom, found_by: usize, triple_index: usize) -> Self {
        Self { object, found_by, triple_index }
    }
}

impl Default for SearchResult {
    fn default() -> Self {
        Self {
            object: SemanticAtom::new([0; 4]),
            found_by: 0,
            triple_index: 0,
        }
    }
}

impl WorkAssignment {
    /// Create a new work assignment
    /// 
    /// # Arguments
    /// * `start_index` - Starting index in the triple store
    /// * `end_index` - Ending index in the triple store
    /// * `target_predicate` - Target predicate to search for
    /// * `hart_id` - Hart ID this assignment is for
    pub const fn new(
        start_index: usize,
        end_index: usize,
        target_predicate: SemanticAtom,
        hart_id: usize,
    ) -> Self {
        Self {
            start_index,
            end_index,
            target_predicate,
            hart_id,
        }
    }
    
    /// Get the number of triples to process
    pub const fn work_size(&self) -> usize {
        self.end_index - self.start_index
    }
    
    /// Check if this assignment is valid
    pub const fn is_valid(&self) -> bool {
        self.start_index < self.end_index && self.hart_id < 4
    }
}

impl Default for WorkAssignment {
    fn default() -> Self {
        Self {
            start_index: 0,
            end_index: 0,
            target_predicate: SemanticAtom::new([0; 4]),
            hart_id: 0,
        }
    }
}

impl Default for TargetAtom {
    fn default() -> Self {
        Self {
            atom: SemanticAtom::new([0; 4]),
            found: false,
            found_by: None,
        }
    }
}
