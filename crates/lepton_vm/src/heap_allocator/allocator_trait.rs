//! The heap used in `Lepton3` require an allocator with garbage
//! collection
//!
//! This file defines the generic trait that these implementations
//! must derive as a common abstraction.

use super::HeapItem;
use crate::values::Value;

/// A heap allocator for Lepton3 heap objects.
///
/// Implementors are responsible for allocation and garbage collection.
///
/// The invariant for GC is that before any allocation, `ensure_capacity` MUST
/// be called with all live `Value` roots, so the collector can update
/// any heap pointers that move during a collection cycle.
pub trait HeapAllocator: Default {
    /// Check whether a GC cycle is required and run one if so,
    /// updating all root pointers in place.
    ///
    /// NOTE: This must always be called before popping a heap-based
    /// value off the stack into a Rust variable, to prevent root
    /// corruption after a collection.
    fn ensure_capacity(&mut self, roots: &mut [&mut Value]);

    /// Allocate a heap item directly, returning its index/pointer.
    ///
    /// Assumes `ensure_capacity` was already called by the caller.
    /// Please read the note for `ensure_capacity`
    fn alloc_raw(&mut self, item: HeapItem) -> usize;

    /// Get the heap item from the index/pointer to the heap
    fn get_item(&self, ptr: usize) -> &HeapItem;

    /// Get a mutable reference to the heap item from the
    /// index/pointer to the heap
    fn get_item_mut(&mut self, ptr: usize) -> &mut HeapItem;
}
