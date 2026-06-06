//! A writer that takes the rust representation of a lepton3 image
//! and outputs the bytes

use alloc::vec::Vec;

use crate::format::{DebugInfo, Function, Image, ObjectType};

use core::num::TryFromIntError;

/// Errors that can occur during the Writing out process
#[derive(Debug)]
pub enum WriteError {
    IntegerOverflow,
}

impl From<TryFromIntError> for WriteError {
    fn from(_: TryFromIntError) -> Self {
        Self::IntegerOverflow
    }
}

impl core::fmt::Display for WriteError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WriteError::IntegerOverflow => {
                write!(f, "value is too large to encode into image format")
            }
        }
    }
}

/// Internal cursor-based writer into a byte buffer
struct Writer {
    data: Vec<u8>,
}

impl Writer {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Write some bytes into the data
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// Write some u8 into the data
    fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    /// Write some u16 into the data
    fn write_u16(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    /// Write some u32 into the data
    fn write_u32(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    /// Try write out some usize length as a u16
    ///
    /// # Errors
    ///
    /// Will return a `WriteError` if the usize does not fit
    /// in a u16.
    fn try_write_usize_u16(&mut self, value: usize) -> Result<(), WriteError> {
        self.write_u16(u16::try_from(value)?);
        Ok(())
    }

    /// Try write out some usize length as a u32
    ///
    /// # Errors
    ///
    /// Will return a `WriteError` if the usize does not fit
    /// in a u32.
    fn try_write_len_u32(&mut self, value: usize) -> Result<(), WriteError> {
        self.write_u32(u32::try_from(value)?);
        Ok(())
    }

    /// Write some string into the data as a u16 length
    /// prefixed bytes
    fn write_string(&mut self, value: &str) -> Result<(), WriteError> {
        let bytes = value.as_bytes();
        self.try_write_usize_u16(bytes.len())?;
        self.write_bytes(bytes);
        Ok(())
    }

    fn finish(self) -> Vec<u8> {
        self.data
    }
}

/// Write a Lepton3 image to raw bytes
///
/// # Errors
///
/// This will error if anything failed during the writing
/// process, for example if an unexpected value occurs or
/// something similar.
///
/// One possible case is that the Image data exceeds the
/// allowed image bounds for the number/length of some things
/// such as strings.
pub fn write(image: &Image) -> Result<Vec<u8>, WriteError> {
    let mut w = Writer::new();

    write_header(&mut w, image);
    write_object_table(&mut w, &image.object_table)?;
    write_function_table(&mut w, &image.function_table)?;
    write_instructions(&mut w, &image.instructions)?;

    if let Some(debug_info) = &image.debug_info {
        write_debug_info(&mut w, debug_info)?;
    }

    Ok(w.finish())
}

// Writes the magic and then all the image header fields out
fn write_header(w: &mut Writer, image: &Image) {
    w.write_bytes(b"LEPTON3");
    w.write_u8(image.header.version_major);
    w.write_u16(image.header.flags.to_raw());
    w.write_u32(image.header.entry_point);
}

/// Writes each object table entry out
fn write_object_table(w: &mut Writer, objects: &[ObjectType]) -> Result<(), WriteError> {
    w.try_write_len_u32(objects.len())?;
    for object in objects {
        w.write_u32(object.field_count);
    }

    Ok(())
}

/// Writes each function table entry out
fn write_function_table(w: &mut Writer, functions: &[Function]) -> Result<(), WriteError> {
    w.try_write_len_u32(functions.len())?;
    for function in functions {
        w.write_u32(function.arg_count);
        w.write_u32(function.local_count);
        w.write_u32(function.instruction_offset);
        w.write_u32(function.instruction_length);
    }

    Ok(())
}

/// Writes all the instructions as a byte array into
/// the image
fn write_instructions(w: &mut Writer, instructions: &[u8]) -> Result<(), WriteError> {
    w.try_write_len_u32(instructions.len())?;
    w.write_bytes(instructions);
    Ok(())
}

/// Writes all the debug information out into the image
fn write_debug_info(w: &mut Writer, debug_info: &DebugInfo) -> Result<(), WriteError> {
    // file table
    w.try_write_len_u32(debug_info.files.len())?;
    for file in &debug_info.files {
        w.write_string(file)?;
    }

    // location table
    w.try_write_len_u32(debug_info.locations.len())?;
    for location in &debug_info.locations {
        w.write_u32(location.instruction_offset);
        w.write_u32(location.file);
        w.write_u32(location.line);
        w.write_u32(location.column);
    }

    Ok(())
}
