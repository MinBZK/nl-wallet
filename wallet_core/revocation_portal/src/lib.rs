use readable_identifier::ReadableIdentifier;

pub mod app;
pub mod revocation_client;
pub mod server;
pub mod settings;
pub mod translations;

pub type DeletionCode = ReadableIdentifier<18>;
