//! Differing underlying possible generators of new tags
//!
//! We aim to provide the end-user the ability to choose
//! between the policy of tag generation
//!
//! These can be selectively enabled using the `tagger_` features,
//! the `bump_gen` option is the default.
//!
//! At least one `tagger_` feature must be enabled for `lepton_vm`
//! to compile.
//!
//! The priority for enabled `tagger_` features is top-down as follows,
//! as only one tag generator variant can be used:
//!
//! 1. `bump_gen`

/// Trait that all generators should implement for unity
mod generator_trait;
pub use generator_trait::TagGenerator;

#[cfg(feature = "tagger_bump_gen")]
mod impl_bump_gen;

#[cfg(feature = "tagger_bump_gen")]
pub use impl_bump_gen::TagBumpGenerator as TagGeneratorImpl;

#[cfg(not(any(feature = "tagger_bump_gen")))]
compile_error! {"At least one tag generator option must be chosen for lepton_vm! enable a `tagger_` feature to pick one."}
