//! The underlying rust representation of all the differing
//! types of values operatable on in Lepton3

use alloc::vec::Vec;

use crate::tagger::TagGenerator;

/// All kinds of values in Lepton3
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Value {
    /// A unit ()
    Unit,

    /// A simple signed integer
    Int(i64),

    /// A simple floating point number
    Float(f64),

    /// A simple boolean
    Bool(bool),

    /// A unique create-once value which should always be unique up to `MAX_u64` (which should realistically be unreachable)
    Tag(Tag),

    /// Integer handle pointing into the heap at an Object
    Object(usize),

    /// Integer handle pointing into the heap at an Array
    Array(usize),
}

/// An opaque unique value in Lepton3
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tag(u64);

impl From<u64> for Tag {
    fn from(value: u64) -> Self {
        Tag(value)
    }
}

impl From<Tag> for u64 {
    fn from(value: Tag) -> Self {
        value.0
    }
}

/// The set of tags identifying each value kind for the `TypeOf` opcode.
pub struct TypeTags {
    pub unit: Tag,
    pub int: Tag,
    pub float: Tag,
    pub boolean: Tag,
    pub tag: Tag,
    pub array: Tag,

    // Mapping of object ids to their tags
    pub object: Vec<Tag>,
}

impl TypeTags {
    /// Creates a new set of type tags
    ///
    /// The objects vec must be a map of all possible objects
    /// to their tag for later lookup
    pub fn new(tagger: &mut impl TagGenerator, obj_tags: Vec<Tag>) -> Self {
        Self {
            unit: tagger.allocate_tag(),
            int: tagger.allocate_tag(),
            float: tagger.allocate_tag(),
            boolean: tagger.allocate_tag(),
            tag: tagger.allocate_tag(),
            array: tagger.allocate_tag(),
            object: obj_tags,
        }
    }
}
