//! `Lepton3` is an experimental free and open-source bytecode virtual
//! machine for the `Fermion3` language.
//!
//! Check out the [repository README](https://github.com/duplessisaurore/lepton3/blob/main/README.md)
//! for more information about the project and join the [Discord](https://discord.gg/wXzj2cqZ3Q) for
//! any discussion.
//!
//! ## Lepton3 Image
//!
//! The `lepton3_image` crate provides parsing for the Lepton3
//! image format which can then be executed by the virtual machine.
//! 
//! The Lepton3 image crate will only permit images whose major
//! version are the same as the virtual machine's current version.
//! 
//! The structure of the image is as follows:
//! 
//! [ HEADER ]
//!   magic:                 [u8; 7]    // "LEPTON3"
//!   version_major:         u8         // major version of lepton3 this is targeted for
//!   flags:                 u16        // flags for the image
//!   entry_point:           u32        // index into function table
//!
//! [ OBJECT TABLE ]
//!  count:                  u32        // total number of objects
//!  for each object type:
//!    field_count:          u32        // number of fields of the object
//!
//! [ FUNCTION TABLE ]
//!  count:                  u32
//!  for each function:
//!    arg_count:            u32        // number of arguments to pass to the function
//!    local_count:          u32        // the maximum number of locals in this function
//!    instruction_offset:   u32        // byte offset into instruction stream
//!    instruction_length:   u32        // byte length of this function's instructions
//!
//! [ INSTRUCTIONS ]
//!  length:                 u32        // total byte length
//!  instructions:           [u8]       // raw instruction stream
//!
//! [ DEBUG INFO ]                      // only present if flags bit 0 set
//!  // file table
//!  file_count:             u32
//!  for each file:
//!    length:               u16
//!    bytes:                [u8]       // utf-8 source file name
//!
//!  // location table
//!  entry_count:            u32
//!  for each entry:
//!    instruction_offset:   u32        // sorted ascending
//!    file:                 u32        // index into string table
//!    line:                 u32        // line into the file
//!    column:               u32        // column into the file

#![warn(clippy::pedantic)]
#![no_std]

/// The above file format for Lepton3, but expressed as
/// Rust structs
pub mod format;

/// Parses the image from raw bytes into the format
#[cfg(feature = "parser")]
pub mod parser;

/// Validates the semantic correctness of an image
#[cfg(feature = "validaor")]
pub mod validator;

/// Serialises the rust struct representation back into an image
#[cfg(feature = "writer")]
pub mod writer;