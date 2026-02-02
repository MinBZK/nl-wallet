//! JWT utilities for strongly-typed JWS creation and verification.
//!
//! ## Overview
//! * This crate provides a thin, type-safe layer on top of `jsonwebtoken`, focusing on strongly-typed payloads and
//!   headers.
//! * Payloads are modelled Rust types and must implement `JwtTyp` (and optionally `JwtSub`) to enforce the `typ` (and
//!   `sub`) fields at verification time.
//! * The crate distinguishes three states of a JWT over the same generic parameters `<T, H>`:
//!   - [`UnverifiedJwt`]: raw serialization received over an untrusted channel. Can be parsed and verified into a
//!     header and payload or a `VerifiedJwt`. If received over a trusted channel it can be "dangerously" parsed without
//!     verifying it.
//!   - [`SignedJwt`]: a freshly signed JWT (with a provided private key). Can be converted it into `UnverifiedJwt` or
//!     directly into a `VerifiedJwt` without verifying it because it was just created.
//!   - [`VerifiedJwt`]: header, payload, and original serialization. Obtained after successful verification of an
//!     `UnverifiedJwt`.
//!
//! ## Headers
//! * [`HeaderWithTyp`](headers::HeaderWithTyp)
//!   - Minimal header with required `alg` and `typ`. Used by default when signing/verifying strongly-typed JWTs.
//!   - [`JwtTyp::TYP` is injected during signing and enforced during verification.
//! * [`HeaderWithX5c<H>`](headers::HeaderWithX5c)
//!   - Header with a required `x5c` field. Use `parse_and_verify_against_trust_anchors` (or
//!     `into_verified_against_trust_anchors`) to validate the certificate chain against the provided trust anchors, and
//!     then verify the JWT using the leaf certificate's public key.
//! * [`HeaderWithJwk<H>`](headers::HeaderWithJwk)
//!   - Header with a required `jwk` field. Used for self-contained verification with `parse_and_verify_with_jwk` or to
//!     pin a specific public key with `parse_and_verify_with_expected_jwk`.
//!
//! ## Traits
//! * [`JwtTyp`]
//!   - Define `const TYP: &'static str`, `"jwt"` by default.
//!   - Verification automatically rejects tokens whose header `typ` does not match.
//! * [`JwtSub`]
//!   - Define `const SUB: &'static str` and use helpers:
//!     - `UnverifiedJwt::parse_and_verify_with_sub` enforces a matching `sub` claim.
//!     - `SignedJwt::sign_with_sub` signs a payload wrapped with the correct `sub` claim.
pub mod confirmation;
pub mod error;
pub mod headers;
pub mod jwk;
pub mod jwt;
pub mod pop;
pub mod wua;

pub use jwt::*;

pub use jsonwebtoken::Algorithm;
pub use jsonwebtoken::Header;
pub use jsonwebtoken::Validation;
