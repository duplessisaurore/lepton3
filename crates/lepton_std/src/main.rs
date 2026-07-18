//! `Lepton3` is an experimental free and open-source bytecode virtual
//! machine for the `Fermion3` language.
//!
//! Check out the [repository README](https://github.com/duplessisaurore/lepton3/blob/main/README.md)
//! for more information about the project and join the [Discord](https://discord.gg/wXzj2cqZ3Q) for
//! any discussion.
//!
//! ## Lepton3 STD
//!
//! The `lepton_std` crate provides a binary for executing lepton
//! image files using the lepton3 virtual machine for systems that support
//! the rust standard, with fs support etc.

use lepton_image::{
    format::Image,
    parser::{self},
    validator,
};
use lepton_vm::{
    heap_allocator::HeapAllocatorImpl,
    tagger::TagGeneratorImpl,
    virtual_machine::{VirtualMachine, VmError},
};
use std::{error::Error, fs, process};

mod capabilities;

fn main() -> Result<(), Box<dyn Error>> {
    // Should be a path to a simple lepton3 binary file
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: {} <image.lp3>", std::env::args().next().unwrap());
        process::exit(1);
    });

    // Read all the bytes from the file
    let bytes = fs::read(&path).unwrap_or_else(|e| {
        eprintln!("error reading {path}: {e}");
        process::exit(1);
    });

    // Parse the image from the bytes.
    let image: Image = parser::parse(&bytes).unwrap_or_else(|e| {
        eprintln!("error parsing image {path}: {e}");
        process::exit(1);
    });

    // Validate the file to ensure it's validity
    validator::validate(&image).unwrap_or_else(|e| {
        eprintln!("error validating image {path}: {e}");
        process::exit(1);
    });

    // Clone the debug info file name
    // so we can keep the file names around for error debugging
    let debug_files = image.debug_info.as_ref().map(|debug| debug.files.clone());

    // Create the virtual machine for execution
    let mut vm = VirtualMachine::new(
        &image,
        capabilities::all(),
        HeapAllocatorImpl::default(),
        TagGeneratorImpl::default(),
    );

    // Run the virtual machine
    match vm.run() {
        Ok(_) => Ok(()),
        Err(VmError::WithTrace { error, trace }) => {
            // Print out runtime stack trace if it's a VMError WithTrace
            eprintln!("runtime error: {error:?}");
            eprintln!("stack trace:");

            // Print out each stack trace frame
            for frame in &trace {
                match &frame.source_location {
                    Some(loc) => {
                        // Look up file name in debug info for source printing
                        let file_name = debug_files
                            .as_ref()
                            .and_then(|files| files.get(loc.file as usize))
                            .map(|s| s.as_str())
                            .unwrap_or("<unknown file>");

                        eprintln!(
                            "  fn[{}] {} {}:{} ({})",
                            frame.function_idx, file_name, loc.line, loc.column, loc.context
                        );
                    }
                    None => eprintln!(
                        "  fn[{}] <no debug info> offset {}",
                        frame.function_idx, frame.instruction_offset
                    ),
                }
            }
            process::exit(1);
        }

        // Other kind of errors, just print them out
        Err(e) => {
            eprintln!("error: {e:?}");
            process::exit(1);
        }
    }
}
