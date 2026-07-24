//! Capability handlers and registration and various
//! other utilities for the Lepton3 virtual machine.

use core::error::Error;

use alloc::{boxed::Box, vec::Vec};
use lepton_image::format::{Image, SourceLocation};

use crate::{
    heap_allocator::HeapAllocatorImpl, tagger::TagGeneratorImpl, values::Value,
    virtual_machine::VirtualMachine,
};

/// A provided capability that the bytecode can invoke via `CallCap`.
///
/// The handler receives a mutable reference to the entirety
/// of the current virtual machine during execution.
///
/// The handler can then access the `stack`/`heap` or anythihngh
/// though this reference.
pub type CapabilityFn<
    'image,
    CS = (),
    SL = SourceLocation,
    H = HeapAllocatorImpl,
    T = TagGeneratorImpl,
    I = Image,
> = fn(virtual_machine: &mut VirtualMachine<'image, CS, SL, H, T, I>) -> Result<(), Box<dyn Error>>;

/// Capability state may hold `Values`.
///
/// This trait provides the ability for the `Capability` values to be added
/// as roots for garbage collection, thus preventing them from being corrupted.
pub trait CapabilityGcRoots {
    /// Append a mutable reference to every `Value` in this capabilities state that
    /// is a root in the heap allocator.
    fn append_roots<'roots>(&'roots mut self, roots: &mut Vec<&'roots mut Value>);
}

impl CapabilityGcRoots for () {
    fn append_roots<'roots>(&'roots mut self, _roots: &mut Vec<&'roots mut Value>) {}
}
