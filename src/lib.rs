//! `Lepton3` is an experimental free and open-source bytecode virtual
//! machine for the `Fermion3` language.
//!
//! Check out the [repository README](https://github.com/duplessisaurore/lepton3/blob/main/README.md)
//! for more information about the project and join the [Discord](https://discord.gg/wXzj2cqZ3Q) for
//! any discussion.
//!
//! The entry point of the virtual machine is through `lepton_vm` and the `lepton_image` crate which
//! parses the image format to be executed by the virtual machine.
//!
//! Capabilities can be added to the virtual machine for rust-lepton interop, see `CapabilityFn`
//!
//! ## This Crate
//!
//! The `lepton` crate is a meta crate that rexports all of the sub-components of lepton into one
//! simpler interface for usage in a compiler to the lepton bytecode or otherwise.

#![warn(clippy::pedantic)]
#![no_std]

// Rexport all commonly used virtual machine elements
pub use lepton_vm::capabilities::CapabilityFn;
pub use lepton_vm::heap_allocator::HeapAllocatorImpl;
pub use lepton_vm::tagger::TagGeneratorImpl;
pub use lepton_vm::virtual_machine::VirtualMachine;

// Rexport all opcodes
pub use lepton_opcodes::Opcode;

// Rexport the image helpers
pub use lepton_image::format;
pub use lepton_image::parser;
pub use lepton_image::validator;
pub use lepton_image::writer;

// Rexport the internal crates
pub use lepton_image;
pub use lepton_opcodes;
pub use lepton_vm;
