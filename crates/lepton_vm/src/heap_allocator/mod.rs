//! Heap allocation and garbage collection for Lepton3.
//!
//! We aim to provide the end-user the ability to choose
//! between the policy of heap allocation and garbage collection
//!
//! These can be selectively enabled using the `heap_allocator_` features,
//! the `cheney` option is the default.
//!
//! At least one `heap_allocator_` feature must be enabled for `lepton_vm`
//! to compile.
//!
//! The priority for enabled `heap_allocator_` features is top-down as follows,
//! as only one tag generator variant can be used:
//!
//! 1. `cheney`

mod allocator_trait;
use alloc::vec::Vec;
pub use allocator_trait::HeapAllocator;

use crate::values::{Tag, Value};

pub enum HeapItem {
    Object { tag: Tag, fields: Vec<Value> },
    Array(Vec<Value>),
    Forwarded(usize),
}

#[cfg(feature = "heap_allocator_cheney")]
mod impl_cheney;

#[cfg(feature = "heap_allocator_cheney")]
pub use impl_cheney::CheneyAllocator as HeapAllocatorImpl;

#[cfg(not(any(feature = "tagger_bump_gen")))]
compile_error!(
    "At least one heap allocator option must be chosen \
                for lepton_vm! Enable a `heap_allocator_` feature to pick one."
);
