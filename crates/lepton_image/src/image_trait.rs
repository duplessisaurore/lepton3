//! This is the image traits, the Virtual Machine though it runs the image
//! format it does not inherently require a `Image` as the user may not want
//! to use fully owned data structures or something.

use alloc::string::String;

use crate::format::{Function, Header, Image, ObjectType, SourceLocation};

/// The image trait, this returns all the data required of a Lepton3 imagae
/// for execution.
pub trait LeptonImage<SL: LeptonSourceLocation = SourceLocation> {
    type File: AsRef<str>;

    /// Returns the header of the image
    fn header(&self) -> &Header;

    /// Returns the object table of the image
    fn object_table(&self) -> &[ObjectType];

    /// Returns the function table of the image
    fn function_table(&self) -> &[Function];

    /// Returns the instructions that are part of the image
    fn instructions(&self) -> &[u8];

    /// Returns the debug info file names of the image if it exists
    fn files(&self) -> Option<&[Self::File]>;

    /// Returns the debug info file locations of the image if it exists
    fn locations(&self) -> Option<&[SL]>;
}

/// Source location trait, this returns all the data required by
/// source locations for debugging
pub trait LeptonSourceLocation {
    type Context: AsRef<str>;

    /// The offset into the instruction stream
    fn instruction_offset(&self) -> u32;

    /// The offset into the file table this source location references
    fn file(&self) -> u32;

    /// The line in the file source location references
    fn line(&self) -> u32;

    /// The column in the file source location references
    fn column(&self) -> u32;

    /// The context in the file source location references
    fn context(&self) -> &Self::Context;
}

impl LeptonImage<SourceLocation> for Image {
    type File = String;

    fn header(&self) -> &Header {
        &self.header
    }

    fn object_table(&self) -> &[ObjectType] {
        self.object_table.as_slice()
    }

    fn function_table(&self) -> &[Function] {
        self.function_table.as_slice()
    }

    fn instructions(&self) -> &[u8] {
        self.instructions.as_slice()
    }

    fn files(&self) -> Option<&[Self::File]> {
        self.debug_info
            .as_ref()
            .map(|debug_info| debug_info.files.as_slice())
    }

    fn locations(&self) -> Option<&[SourceLocation]> {
        self.debug_info
            .as_ref()
            .map(|debug_info| debug_info.locations.as_slice())
    }
}

impl LeptonSourceLocation for SourceLocation {
    type Context = String;

    /// The offset into the instruction stream
    fn instruction_offset(&self) -> u32 {
        self.instruction_offset
    }

    /// The offset into the file table this source location references
    fn file(&self) -> u32 {
        self.file
    }

    /// The line in the file source location references
    fn line(&self) -> u32 {
        self.line
    }

    /// The column in the file source location references
    fn column(&self) -> u32 {
        self.column
    }

    /// The context in the file source location references
    fn context(&self) -> &Self::Context {
        &self.context
    }
}
