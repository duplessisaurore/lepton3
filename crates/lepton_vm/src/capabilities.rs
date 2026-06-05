//! Capability handlers and registration and various
//! other utilities for the Lepton3 virtual machine.

use core::error::Error;

use alloc::{boxed::Box, vec::Vec};

use crate::values::Value;

/// A provided capability that the bytecode can invoke via `CallCap`.
///
/// The handler receives the current value stack (mutable) and may push or
/// pop values as needed. It also receives a mutable reference to the heap
/// allocator and tag generator so it can allocate objects if necessary.
pub type CapabilityFn<H, T> =
    fn(stack: &mut Vec<Value>, heap: &mut H, tagger: &mut T) -> Result<(), Box<dyn Error>>;
