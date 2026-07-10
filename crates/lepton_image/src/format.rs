//! The format of the image expressed as rust
//! structures rather than written in english in the lib.rs

use alloc::{string::String, vec::Vec};

use crate::flags::ImageFlags;

/// The magic bytes expected at the beginning of a Lepton3 image
pub const MAGIC: &[u8] = b"LEPTON3";

/// A const function that lets us parse some string
/// into a u8 at compile time.
const fn parse_u8(s: &str) -> u8 {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut num: u8 = 0;

    while i < bytes.len() {
        let b = bytes[i];
        assert!(b >= b'0' && b <= b'9', "Non-digit in version string");
        num = num * 10 + (b - b'0');
        i += 1;
    }
    num
}

/// The current VM major version, used to check image compatibility
///
/// This is taken from the `lepton_image` cargo.toml version. This should
/// only be changed rarely if a highly breaking change is made, as the VM
/// only executes things with the same major version!
pub const VM_MAJOR_VERSION: u8 = parse_u8(env!("CARGO_PKG_VERSION_MAJOR"));

/// A full Lepton3 bytecode image
pub struct Image {
    pub header: Header,
    pub object_table: Vec<ObjectType>,
    pub function_table: Vec<Function>,
    pub instructions: Vec<u8>,
    pub debug_info: Option<DebugInfo>,
}

/// Image header
pub struct Header {
    pub version_major: u8,
    pub flags: ImageFlags,
    pub entry_point: u32,
}

/// A single object type definition
pub struct ObjectType {
    pub field_count: u32,
}

/// A single function definition
pub struct Function {
    pub arg_count: u32,
    pub local_count: u32,
    pub instruction_offset: u32,
    pub instruction_length: u32,
}

/// Debug information that can be attached
/// to link back to a source location
pub struct DebugInfo {
    pub files: Vec<String>,
    pub locations: Vec<SourceLocation>,
}

/// A source location that links an instruction
/// to a line/column in a file
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub instruction_offset: u32,
    pub file: u32,
    pub line: u32,
    pub column: u32,
    pub context: String,
}
