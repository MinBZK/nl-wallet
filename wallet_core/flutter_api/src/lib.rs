#![recursion_limit = "256"]

pub mod api;
mod errors;
#[expect(clippy::cast_lossless)]
#[rustfmt::skip]
mod frb_generated;
mod logging;
mod models;
mod sentry;
