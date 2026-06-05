//! `Lepton3` is an experimental free and open-source bytecode virtual
//! machine for the `Fermion3` language.
//!
//! Check out the [repository README](https://github.com/duplessisaurore/lepton3/blob/main/README.md)
//! for more information about the project and join the [Discord](https://discord.gg/wXzj2cqZ3Q) for
//! any discussion.
//!
//! ## Lepton3 Virtual Machine
//!
//! The `lepton_vm` crate provides the actual virtual machine
//! responsible for execution of the Lepton3 bytecode.

#![warn(clippy::pedantic)]
#![no_std]

/// We need allocations for arrays and
/// various other types on the heap.
extern crate alloc;

/// All types of values operatable on in the Lepton3
/// bytecode.
pub mod values;

/// The actual bytecode interpreter of an image
pub mod virtual_machine;

/// Tag generator implementation
pub mod tagger;

// Heap allocator implementation
pub mod heap_allocator;

/// Capabilities provided to the VM
pub mod capabilities;
