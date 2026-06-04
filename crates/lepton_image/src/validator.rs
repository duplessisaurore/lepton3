//! A validator that validates the semantic correctness
//! of a Lepton3 image.

use crate::format::{DebugInfo, Image, VM_MAJOR_VERSION};

/// Errors that can occur during validation
#[derive(Debug)]
pub enum ValidationError {
    /// The image's major version does not match the VM's major version
    VersionMismatch { image: u8, vm: u8 },
    /// The entry point index is out of bounds
    InvalidEntryPoint { index: u32, function_count: usize },
    /// A function's instruction range exceeds the instruction stream
    FunctionOutOfBounds { function_index: usize },
    /// A function's `local_count` is less than its `arg_count`
    LocalCountTooSmall { function_index: usize },
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
///
/// # Errors
///
/// This will error if the validation fails
/// for the parsed Lepton3 image in any way.
pub fn validate(image: &Image) -> Result<(), ValidationError> {
    validate_version(image)?;
    validate_entry_point(image)?;
    validate_functions(image)?;

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
        let start = u64::from(function.instruction_offset);
        let end = start + u64::from(function.instruction_length);
        if end > stream_len {
            return Err(ValidationError::FunctionOutOfBounds { function_index: i });
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
        if let Some(prev) = last_offset
            && location.instruction_offset < prev
        {
            return Err(ValidationError::LocationTableUnsorted { entry: i });
        }
        last_offset = Some(location.instruction_offset);
    }

    Ok(())
}
