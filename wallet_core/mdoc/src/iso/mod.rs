//! This module contains data structures defined in the ISO 18013-5 spec, divided into four submodules.
//!
//! Some conventions used here:
//! - Type aliases are used where possible, in particular when no methods need to be attached to the data structure.
//! - For some structs, the spec demands that they should be serialized as a sequence (i.e., without the field names)
//!   or as a map with integer values (also without the field names). In this case, we define an associated struct
//!   whose name ends on `Keyed` which does use field names. This allows us to refer to the contents of such data
//!   structures by name instead of by numbers. We transform them into the form required by the spec using the
//!   [`CborSeq`](crate::serialization::CborSeq) and [`CborIntMap`](crate::serialization::CborIntMap) wrappers.
//! - Some CBOR data structures contain other data structures not directly, but instead their CBOR-serialized bytes.
//!   For this the [`TaggedBytes`](crate::TaggedBytes) wrapper is used.

pub mod basic_sa_ext;

pub mod credentials;
pub use credentials::*;

pub mod device_retrieval;
pub use device_retrieval::*;

pub mod disclosure;
pub use disclosure::*;

pub mod engagement;
pub use engagement::*;

pub mod issuance;
pub use issuance::*;
