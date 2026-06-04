//! A validator that validates the semantic correctness
//! of a Lepton3 image.

use crate::format::{DebugInfo, Image, VM_MAJOR_VERSION};
use alloc::vec::Vec;
use hashbrown::HashSet;
use lepton_opcodes::Opcode;

/// Errors that can occur during validation
#[derive(Debug)]
pub enum ValidationError {
    /// The image's major version does not match the VM's major version
    VersionMismatch { image: u8, vm: u8 },
    /// The entry point index is out of bounds
    InvalidEntryPoint { index: u32, function_count: usize },
    /// A function's instruction range exceeds the instruction stream
    FunctionOutOfBounds { function_index: usize },
    /// A function's local_count is less than its arg_count
    LocalCountTooSmall { function_index: usize },
    /// An unknown opcode was encountered
    UnknownOpcode { opcode: u8, offset: u32 },
    /// An instruction's operand bytes exceed the instruction stream
    InstructionOutOfBounds { offset: u32 },
    /// A jump offset is out of bounds
    InvalidJumpOffset { offset: u32, target: u32 },
    /// A Call instruction references an invalid function index
    InvalidFunctionIndex { offset: u32, index: u32 },
    /// An ObjectNew instruction references an invalid object table index
    InvalidObjectIndex { offset: u32, index: u32 },
    /// A Load/Store instruction references an index beyond local_count
    InvalidLocalIndex {
        offset: u32,
        index: u32,
        function_index: usize,
    },
    /// A debug info file index is out of bounds in the location table
    InvalidFileIndex { entry: usize },
    /// The location table is not sorted by instruction offset
    LocationTableUnsorted { entry: usize },
}

impl core::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ValidationError::VersionMismatch { image, vm } => write!(
                f,
                "image major version {image} does not match VM major version {vm}"
            ),
            ValidationError::InvalidEntryPoint {
                index,
                function_count,
            } => write!(
                f,
                "entry point index {index} into function table is out of bounds, function count is {function_count}"
            ),
            ValidationError::FunctionOutOfBounds { function_index } => write!(
                f,
                "function {function_index} instruction range exceeds total instruction stream size"
            ),
            ValidationError::LocalCountTooSmall { function_index } => write!(
                f,
                "function {function_index} local_count is less than arg_count"
            ),
            ValidationError::UnknownOpcode { opcode, offset } => {
                write!(f, "unknown opcode 0x{opcode:02X} at offset {offset}")
            }
            ValidationError::InstructionOutOfBounds { offset } => write!(
                f,
                "instruction at offset {offset} exceeds total instruction stream size"
            ),
            ValidationError::InvalidJumpOffset { offset, target } => {
                write!(f, "jump at offset {offset} targets invalid offset {target}")
            }
            ValidationError::InvalidFunctionIndex { offset, index } => write!(
                f,
                "call at offset {offset} references invalid function index {index}"
            ),
            ValidationError::InvalidObjectIndex { offset, index } => write!(
                f,
                "object instruction at offset {offset} references invalid object index {index}"
            ),
            ValidationError::InvalidLocalIndex {
                offset,
                index,
                function_index,
            } => write!(
                f,
                "local instruction at offset {offset} references invalid local index {index} in function {function_index}"
            ),
            ValidationError::InvalidFileIndex { entry } => write!(
                f,
                "debug location entry {entry} references invalid file index"
            ),
            ValidationError::LocationTableUnsorted { entry } => {
                write!(f, "debug location table is not sorted at entry {entry}")
            }
        }
    }
}

/// Validate a parsed Lepton3 image
pub fn validate(image: &Image) -> Result<(), ValidationError> {
    validate_version(image)?;
    validate_entry_point(image)?;
    validate_functions(image)?;
    validate_instructions(image)?;

    if let Some(debug_info) = &image.debug_info {
        validate_debug_info(debug_info)?;
    }

    Ok(())
}

fn validate_version(image: &Image) -> Result<(), ValidationError> {
    // check the major version matches image
    if image.header.version_major != VM_MAJOR_VERSION {
        return Err(ValidationError::VersionMismatch {
            image: image.header.version_major,
            vm: VM_MAJOR_VERSION,
        });
    }
    Ok(())
}

fn validate_entry_point(image: &Image) -> Result<(), ValidationError> {
    // check image header entry point falls into function table length
    let index = image.header.entry_point as usize;
    if index >= image.function_table.len() {
        return Err(ValidationError::InvalidEntryPoint {
            index: image.header.entry_point,
            function_count: image.function_table.len(),
        });
    }
    Ok(())
}

fn validate_functions(image: &Image) -> Result<(), ValidationError> {
    let stream_len = image.instructions.len() as u64;

    for (i, function) in image.function_table.iter().enumerate() {
        // local_count must be at least arg_count
        if function.local_count < function.arg_count {
            return Err(ValidationError::LocalCountTooSmall { function_index: i });
        }

        // instruction range must be within the stream
        let start = function.instruction_offset as u64;
        let end = start + function.instruction_length as u64;
        if end > stream_len {
            return Err(ValidationError::FunctionOutOfBounds { function_index: i });
        }
    }

    Ok(())
}

fn validate_instructions(image: &Image) -> Result<(), ValidationError> {
    let stream = &image.instructions;

    // validate the instructions in each function are legitimate and valid
    // to the best we can from the image only.
    for (fn_index, function) in image.function_table.iter().enumerate() {
        let start = function.instruction_offset;
        let end = start + function.instruction_length;
        let fn_stream = &stream[start as usize..end as usize];

        validate_function_instructions(fn_stream, start, fn_index, function.local_count, image)?;
    }

    Ok(())
}

fn validate_function_instructions(
    fn_stream: &[u8],
    base_offset: u32,
    fn_index: usize,
    local_count: u32,
    image: &Image,
) -> Result<(), ValidationError> {
    let fn_len = fn_stream.len();

    // parse all opcodes and their offset for further opcode
    // specific checking of offsets and things later.
    let mut instructions: Vec<(u32, Opcode)> = Vec::new();
    let mut c = 0usize;

    // parse each opcode in the function length
    while c < fn_len {
        let local_offset = c as u32;
        let byte = fn_stream[c];

        // try parse the opcode to validate it's correcntess
        let opcode = Opcode::try_from(byte).map_err(|_| ValidationError::UnknownOpcode {
            opcode: byte,
            offset: base_offset + local_offset,
        })?;

        // check the bounds of every opcode fall into the function size
        let next = c + 1 + opcode.operand_size() as usize;
        if next > fn_len {
            return Err(ValidationError::InstructionOutOfBounds {
                offset: base_offset + local_offset,
            });
        }

        instructions.push((local_offset, opcode));
        c = next;
    }

    // Collect valid jump targets from the instruction offsets
    let valid_offsets: HashSet<u32> = instructions.iter().map(|(offset, _)| *offset).collect();

    // Now validate each opcode to check the offsets are correct.
    for (local_offset, opcode) in &instructions {
        let abs_offset = base_offset + local_offset;

        // operand bytes start immediately after the opcode byte
        let operand = (local_offset + 1) as usize;

        match opcode {
            // Validate the jump target falls within the valid offsets
            Opcode::Jump | Opcode::JumpIfTrue | Opcode::JumpIfFalse | Opcode::Try => {
                let target = read_u32(fn_stream, operand);
                if !valid_offsets.contains(&target) {
                    return Err(ValidationError::InvalidJumpOffset {
                        offset: abs_offset,
                        target,
                    });
                }
            }

            // Validate the call function index falls in the function table
            Opcode::Call => {
                let index = read_u32(fn_stream, operand);
                if index as usize >= image.function_table.len() {
                    return Err(ValidationError::InvalidFunctionIndex {
                        offset: abs_offset,
                        index,
                    });
                }
            }

            // Validate the object new index falls into the object table
            Opcode::ObjectNew => {
                let index = read_u32(fn_stream, operand);
                if index as usize >= image.object_table.len() {
                    return Err(ValidationError::InvalidObjectIndex {
                        offset: abs_offset,
                        index,
                    });
                }
            }

            // Validate load/store grabs from a valid index into the local table.
            Opcode::Load | Opcode::Store => {
                let index = read_u32(fn_stream, operand);
                if index >= local_count {
                    return Err(ValidationError::InvalidLocalIndex {
                        offset: abs_offset,
                        index,
                        function_index: fn_index,
                    });
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn validate_debug_info(debug_info: &DebugInfo) -> Result<(), ValidationError> {
    let file_count = debug_info.files.len();
    let mut last_offset: Option<u32> = None;

    for (i, location) in debug_info.locations.iter().enumerate() {
        // file index must be valid
        if location.file as usize >= file_count {
            return Err(ValidationError::InvalidFileIndex { entry: i });
        }

        // location table must be sorted ascending by instruction offset
        if let Some(prev) = last_offset {
            if location.instruction_offset < prev {
                return Err(ValidationError::LocationTableUnsorted { entry: i });
            }
        }
        last_offset = Some(location.instruction_offset);
    }

    Ok(())
}

/// Read a u32 from a byte slice at a given offset (little-endian)
fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}
