//! A simple sequentially increasing "bump"
//! generator as the tag allocator
//!
//! This ensures uniqueness by ensuring each
//! generated tag is `prior_tag` + `bump_count` and never reusing
//! an older tag.
//!
//! This will only permit up to `MAX_U64` tags.

use crate::{tagger::generator_trait::TagGenerator, values::Tag};

pub struct TagBumpGenerator {
    prior_tag: Tag,
    bump_count: u64,
}

impl TagGenerator for TagBumpGenerator {
    fn allocate_tag(&mut self) -> Tag {
        let next = u64::from(self.prior_tag) + self.bump_count;
        let new_tag = Tag::from(next);
        self.prior_tag = new_tag;
        new_tag
    }
}

impl Default for TagBumpGenerator {
    fn default() -> Self {
        Self {
            // A default bump generator begins from a tag of 0
            // with a bump count of only 1
            prior_tag: Tag::from(0u64),
            bump_count: 1u64,
        }
    }
}
