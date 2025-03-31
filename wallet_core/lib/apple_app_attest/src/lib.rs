pub use self::app_identifier::*;
pub use self::assertion::*;
pub use self::attestation::*;
pub use self::auth_data::*;
pub use self::certificates::*;
pub use self::root_ca::*;

#[cfg(feature = "mock")]
pub use self::attestation::mock::MockAttestationCa;

pub mod passkey_types {
    pub use passkey_types::ctap2::Aaguid;
    pub use passkey_types::ctap2::AttestedCredentialData;
    pub use passkey_types::ctap2::AuthenticatorData;
    pub use passkey_types::ctap2::Flags;
}

mod app_identifier;
mod assertion;
mod attestation;
mod auth_data;
mod certificates;
mod root_ca;
