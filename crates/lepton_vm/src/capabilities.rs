//! Capability handlers and registration and various
//! other utilities for the Lepton3 virtual machine.

use core::error::Error;

use alloc::boxed::Box;
use lepton_image::format::{Image, SourceLocation};

use crate::{
    heap_allocator::HeapAllocatorImpl, tagger::TagGeneratorImpl, virtual_machine::VirtualMachine,
};

/// A provided capability that the bytecode can invoke via `CallCap`.
///
/// The handler receives a mutable reference to the entirety
/// of the current virtual machine during execution.
///
/// The handler can then access the `stack`/`heap` or anythihngh
/// though this reference.
pub type CapabilityFn<SL = SourceLocation, H = HeapAllocatorImpl, T = TagGeneratorImpl, I = Image> =
    fn(virtual_machine: &mut VirtualMachine<SL, H, T, I>) -> Result<(), Box<dyn Error>>;
