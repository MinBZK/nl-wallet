cfg_if::cfg_if! {
    if #[cfg(any(feature = "issuance", feature = "disclosure"))] {
        pub mod log_requests;
        pub mod server;
        pub mod keys;
    }
}
cfg_if::cfg_if! {
    if #[cfg(any(feature = "issuance", feature = "disclosure", feature = "postgres"))] {
        pub mod settings;
        pub mod store;
    }
}
cfg_if::cfg_if! {
    if #[cfg(feature = "issuance")] {
        pub mod issuer;
        pub mod pid;
    }
}
#[cfg(feature = "disclosure")]
pub mod verifier;

#[cfg(feature = "postgres")]
pub mod entity;
