//! A parser that takes the raw bytes of a lepton3 image
//! and outputs the Rust struct representation

use alloc::{string::String, vec::Vec};

use crate::{
    flags::ImageFlags,
    format::{DebugInfo, Function, Header, Image, ObjectType, SourceLocation},
};

/// Errors that can occur during parsing
#[derive(Debug)]
pub enum ParseError {
    /// The image is too short to contain the expected data
    UnexpectedEof,
    /// The magic bytes do not match "LEPTON3"
    InvalidMagic,
    /// A string in the image is not valid UTF-8
    InvalidUtf8,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseError::UnexpectedEof => write!(f, "unexpected end of image data"),
            ParseError::InvalidMagic => write!(f, "invalid magic bytes, expected LEPTON3"),
            ParseError::InvalidUtf8 => write!(f, "invalid utf-8 in image string data"),
        }
    }
}

/// Internal cursor-based reader over a byte slice
struct Reader<'a> {
    data: &'a [u8],
    cursor: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    /// Returns the ramining number of bytes left in
    /// the data from the current cursor position
    fn remaining(&self) -> usize {
        self.data.len() - self.cursor
    }

    /// Reads a certain number of bytes from the reader, advancing
    /// the cursor by the number of bytes if successful
    ///
    /// # Errors
    ///
    /// If there is not the specified number of bytes left in the data,
    /// an `UnexpectedEof` error will be returned
    fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], ParseError> {
        if self.remaining() < count {
            return Err(ParseError::UnexpectedEof);
        }
        let slice = &self.data[self.cursor..self.cursor + count];
        self.cursor += count;
        Ok(slice)
    }

    /// Expects to read a u8 from the data
    fn read_u8(&mut self) -> Result<u8, ParseError> {
        Ok(self.read_bytes(1)?[0])
    }

    /// Expects to read a u16 from the data
    fn read_u16(&mut self) -> Result<u16, ParseError> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// Expects to read a u32 from the data
    fn read_u32(&mut self) -> Result<u32, ParseError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Expects to read a UTF-8 string from the data
    fn read_string(&mut self) -> Result<String, ParseError> {
        let length = self.read_u16()? as usize;
        let bytes = self.read_bytes(length)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| ParseError::InvalidUtf8)
    }
}

/// Fully parse a Lepton3 image from raw bytes
///
/// # Errors
///
/// If there is an issue when parsing this file, say
/// due to some issue with a string or an unexpected EOF
/// then a `ParseError` will be returned.
pub fn parse(bytes: &[u8]) -> Result<Image, ParseError> {
    let mut r = Reader::new(bytes);

    // Parse each component
    let header = parse_header(&mut r)?;
    let object_table = parse_object_table(&mut r)?;
    let function_table = parse_function_table(&mut r)?;
    let instructions = parse_instructions(&mut r)?;

    // Parse debug info if header flag is set
    let debug_info = if header.flags.has(ImageFlags::DEBUG_INFO) {
        Some(parse_debug_info(&mut r)?)
    } else {
        None
    };

    Ok(Image {
        header,
        object_table,
        function_table,
        instructions,
        debug_info,
    })
}

/// Parses the full Lepton3 header from the image.
fn parse_header(r: &mut Reader) -> Result<Header, ParseError> {
    // Expect the magic bytes at the start
    let magic = r.read_bytes(7)?;
    if magic != b"LEPTON3" {
        return Err(ParseError::InvalidMagic);
    }

    let version_major = r.read_u8()?;
    let flags = r.read_u16()?;
    let entry_point = r.read_u32()?;

    Ok(Header {
        version_major,
        flags: ImageFlags::from_raw(flags),
        entry_point,
    })
}

/// Parses the full Object table from a Lepton3 image
fn parse_object_table(r: &mut Reader) -> Result<Vec<ObjectType>, ParseError> {
    let count = r.read_u32()? as usize;
    let mut objects = Vec::with_capacity(count);

    for _ in 0..count {
        let field_count = r.read_u32()?;
        objects.push(ObjectType { field_count });
    }

    Ok(objects)
}

/// Parses the full function table from the Lepton3 image
fn parse_function_table(r: &mut Reader) -> Result<Vec<Function>, ParseError> {
    let count = r.read_u32()? as usize;
    let mut functions = Vec::with_capacity(count);

    for _ in 0..count {
        let arg_count = r.read_u32()?;
        let local_count = r.read_u32()?;
        let instruction_offset = r.read_u32()?;
        let instruction_length = r.read_u32()?;

        functions.push(Function {
            arg_count,
            local_count,
            instruction_offset,
            instruction_length,
        });
    }

    Ok(functions)
}

/// Parse all the instructions from the Lepton3 image
fn parse_instructions(r: &mut Reader) -> Result<Vec<u8>, ParseError> {
    let length = r.read_u32()? as usize;
    let bytes = r.read_bytes(length)?;
    Ok(bytes.to_vec())
}

/// Parses the debug information from the Lepton3 image
fn parse_debug_info(r: &mut Reader) -> Result<DebugInfo, ParseError> {
    // file table
    let file_count = r.read_u32()? as usize;
    let mut files = Vec::with_capacity(file_count);
    for _ in 0..file_count {
        files.push(r.read_string()?);
    }

    // location table
    let entry_count = r.read_u32()? as usize;
    let mut locations = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        let instruction_offset = r.read_u32()?;
        let file = r.read_u32()?;
        let line = r.read_u32()?;
        let column = r.read_u32()?;
        locations.push(SourceLocation {
            instruction_offset,
            file,
            line,
            column,
        });
    }

    Ok(DebugInfo { files, locations })
}
