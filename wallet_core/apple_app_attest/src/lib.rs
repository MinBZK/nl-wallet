pub use self::{app_identifier::*, assertion::*, attestation::*, auth_data::*, certificates::*, root_ca::*};

pub mod passkey_types {
    pub use passkey_types::ctap2::{Aaguid, AttestedCredentialData, AuthenticatorData, Flags};
}

mod app_identifier;
mod assertion;
mod attestation;
mod auth_data;
mod certificates;
mod root_ca;
