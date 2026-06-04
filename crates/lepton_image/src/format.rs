//! The format of the image expressed as rust
//! structures rather than written in english in the lib.rs

use alloc::{string::String, vec::Vec};

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
    pub flags: u16,
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
pub struct SourceLocation {
    pub instruction_offset: u32,
    pub file: u32,
    pub line: u32,
    pub column: u32,
}
