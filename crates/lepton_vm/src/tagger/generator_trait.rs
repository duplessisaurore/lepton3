//! The tags used in `Lepton3` require a generator
//! that produces always unique tags as a baseline requirement
//!
//! This file defines the generic trait that these implementations
//! must derive as a common abstraction.

use crate::values::Tag;

/// A tag generator, at a baseline this
/// MUST ensure the generation of unique tags.
pub trait TagGenerator: Default {
    /// Allocate a new tag, due to the
    /// unique requirements, a tag can never
    /// be unallocated to prevent accidental insanity
    /// corruption around tag reuse
    ///
    /// Mutable access is given to generator as the
    /// generator may require holding onto mutable state
    /// to determine the next tag.
    fn allocate_tag(&mut self) -> Tag;
}
