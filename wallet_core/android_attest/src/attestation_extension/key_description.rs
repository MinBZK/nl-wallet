//! This file implements a schema that supports the following versions of the Android Key Attestation Extension schema:
//!
//! - 3
//! - 4
//! - 100
//! - 200
//! - 300

pub use rasn::types::Integer;
pub use rasn::types::OctetString;
pub use rasn::types::SetOf;
use rasn::AsnType;
use rasn::Decode;
use rasn::Decoder;
#[cfg(feature = "encode")]
use rasn::Encoder;

// KeyDescription ::= SEQUENCE {
//     attestationVersion  INTEGER,
//     attestationSecurityLevel  SecurityLevel,
//     keyMintVersion  INTEGER,
//     keyMintSecurityLevel  SecurityLevel,
//     attestationChallenge  OCTET_STRING,
//     uniqueId  OCTET_STRING,
//     softwareEnforced  AuthorizationList,
//     hardwareEnforced  AuthorizationList,
// }
#[derive(Debug, Clone, PartialEq, Eq, AsnType, Decode)]
#[cfg_attr(feature = "encode", derive(rasn::Encode))]
pub struct KeyDescription {
    pub attestation_version: Integer,
    pub attestation_security_level: SecurityLevel,
    pub key_mint_version: Integer,
    pub key_mint_security_level: SecurityLevel,
    pub attestation_challenge: OctetString,
    pub unique_id: OctetString,
    pub software_enforced: AuthorizationList,
    pub hardware_enforced: AuthorizationList,
}

// SecurityLevel ::= ENUMERATED {
//     Software  (0),
//     TrustedEnvironment  (1),
//     StrongBox  (2),
// }
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsnType, Decode)]
#[cfg_attr(feature = "encode", derive(rasn::Encode))]
#[rasn(enumerated)]
pub enum SecurityLevel {
    Software,
    TrustedEnvironment,
    StrongBox,
}

// AuthorizationList ::= SEQUENCE {
//     purpose  [1] EXPLICIT SET OF INTEGER OPTIONAL,
//     algorithm  [2] EXPLICIT INTEGER OPTIONAL,
//     keySize  [3] EXPLICIT INTEGER OPTIONAL,
//     digest  [5] EXPLICIT SET OF INTEGER OPTIONAL,
//     padding  [6] EXPLICIT SET OF INTEGER OPTIONAL,
//     ecCurve  [10] EXPLICIT INTEGER OPTIONAL,
//     rsaPublicExponent  [200] EXPLICIT INTEGER OPTIONAL,
//     mgfDigest  [203] EXPLICIT SET OF INTEGER OPTIONAL,
//     rollbackResistance  [303] EXPLICIT NULL OPTIONAL,
//     earlyBootOnly  [305] EXPLICIT NULL OPTIONAL,
//     activeDateTime  [400] EXPLICIT INTEGER OPTIONAL,
//     originationExpireDateTime  [401] EXPLICIT INTEGER OPTIONAL,
//     usageExpireDateTime  [402] EXPLICIT INTEGER OPTIONAL,
//     usageCountLimit  [405] EXPLICIT INTEGER OPTIONAL,
//     noAuthRequired  [503] EXPLICIT NULL OPTIONAL,
//     userAuthType  [504] EXPLICIT INTEGER OPTIONAL,
//     authTimeout  [505] EXPLICIT INTEGER OPTIONAL,
//     allowWhileOnBody  [506] EXPLICIT NULL OPTIONAL,
//     trustedUserPresenceRequired  [507] EXPLICIT NULL OPTIONAL,
//     trustedConfirmationRequired  [508] EXPLICIT NULL OPTIONAL,
//     unlockedDeviceRequired  [509] EXPLICIT NULL OPTIONAL,
//     allApplications  [600] EXPLICIT NULL OPTIONAL,
//     creationDateTime  [701] EXPLICIT INTEGER OPTIONAL,
//     origin  [702] EXPLICIT INTEGER OPTIONAL,
//     rootOfTrust  [704] EXPLICIT RootOfTrust OPTIONAL,
//     osVersion  [705] EXPLICIT INTEGER OPTIONAL,
//     osPatchLevel  [706] EXPLICIT INTEGER OPTIONAL,
//     attestationApplicationId  [709] EXPLICIT OCTET_STRING OPTIONAL,
//     attestationIdBrand  [710] EXPLICIT OCTET_STRING OPTIONAL,
//     attestationIdDevice  [711] EXPLICIT OCTET_STRING OPTIONAL,
//     attestationIdProduct  [712] EXPLICIT OCTET_STRING OPTIONAL,
//     attestationIdSerial  [713] EXPLICIT OCTET_STRING OPTIONAL,
//     attestationIdImei  [714] EXPLICIT OCTET_STRING OPTIONAL,
//     attestationIdMeid  [715] EXPLICIT OCTET_STRING OPTIONAL,
//     attestationIdManufacturer  [716] EXPLICIT OCTET_STRING OPTIONAL,
//     attestationIdModel  [717] EXPLICIT OCTET_STRING OPTIONAL,
//     vendorPatchLevel  [718] EXPLICIT INTEGER OPTIONAL,
//     bootPatchLevel  [719] EXPLICIT INTEGER OPTIONAL,
//     deviceUniqueAttestation  [720] EXPLICIT NULL OPTIONAL,
//     attestationIdSecondImei  [723] EXPLICIT OCTET_STRING OPTIONAL,
// }
#[derive(Debug, Clone, PartialEq, Eq, Default, AsnType, Decode)]
#[cfg_attr(feature = "encode", derive(rasn::Encode))]
pub struct AuthorizationList {
    #[rasn(tag(explicit(1)))]
    pub purpose: Option<SetOf<Integer>>,
    #[rasn(tag(explicit(2)))]
    pub algorithm: Option<Integer>,
    #[rasn(tag(explicit(3)))]
    pub key_size: Option<Integer>,
    #[rasn(tag(explicit(5)))]
    pub digest: Option<SetOf<Integer>>,
    #[rasn(tag(explicit(6)))]
    pub padding: Option<SetOf<Integer>>,
    #[rasn(tag(explicit(10)))]
    pub ec_curve: Option<Integer>,
    #[rasn(tag(explicit(200)))]
    pub rsa_public_exponent: Option<Integer>,
    #[rasn(tag(explicit(203)))]
    pub mgf_digest: Option<SetOf<Integer>>,
    #[rasn(tag(explicit(303)))]
    pub rollback_resistance: Option<()>,
    #[rasn(tag(explicit(305)))]
    pub early_boot_only: Option<()>,
    #[rasn(tag(explicit(400)))]
    pub active_date_time: Option<Integer>,
    #[rasn(tag(explicit(401)))]
    pub origination_expire_date_time: Option<Integer>,
    #[rasn(tag(explicit(402)))]
    pub usage_expire_date_time: Option<Integer>,
    #[rasn(tag(explicit(405)))]
    pub usage_count_limit: Option<Integer>,
    #[rasn(tag(explicit(503)))]
    pub no_auth_required: Option<()>,
    #[rasn(tag(explicit(504)))]
    pub user_auth_type: Option<Integer>,
    #[rasn(tag(explicit(505)))]
    pub auth_timeout: Option<Integer>,
    #[rasn(tag(explicit(506)))]
    pub allow_while_on_body: Option<()>,
    #[rasn(tag(explicit(507)))]
    pub trusted_user_presence_required: Option<()>,
    #[rasn(tag(explicit(508)))]
    pub trusted_confirmation_required: Option<()>,
    #[rasn(tag(explicit(509)))]
    pub unlocked_device_required: Option<()>,
    // kept for backwards compatibility reasons, this field is removed in version 100
    #[rasn(tag(explicit(600)))]
    pub all_applications: Option<()>,
    #[rasn(tag(explicit(701)))]
    pub creation_date_time: Option<Integer>,
    #[rasn(tag(explicit(702)))]
    pub origin: Option<Integer>,
    #[rasn(tag(explicit(704)))]
    pub root_of_trust: Option<RootOfTrust>,
    #[rasn(tag(explicit(705)))]
    pub os_version: Option<Integer>,
    #[rasn(tag(explicit(706)))]
    pub os_patch_level: Option<Integer>,
    #[rasn(tag(explicit(709)))]
    pub attestation_application_id: Option<OctetString>,
    #[rasn(tag(explicit(710)))]
    pub attestation_id_brand: Option<OctetString>,
    #[rasn(tag(explicit(711)))]
    pub attestation_id_device: Option<OctetString>,
    #[rasn(tag(explicit(712)))]
    pub attestation_id_product: Option<OctetString>,
    #[rasn(tag(explicit(713)))]
    pub attestation_id_serial: Option<OctetString>,
    #[rasn(tag(explicit(714)))]
    pub attestation_id_imei: Option<OctetString>,
    #[rasn(tag(explicit(715)))]
    pub attestation_id_meid: Option<OctetString>,
    #[rasn(tag(explicit(716)))]
    pub attestation_id_manufacturer: Option<OctetString>,
    #[rasn(tag(explicit(717)))]
    pub attestation_id_model: Option<OctetString>,
    #[rasn(tag(explicit(718)))]
    pub vendor_patch_level: Option<Integer>,
    #[rasn(tag(explicit(719)))]
    pub boot_patch_level: Option<Integer>,
    #[rasn(tag(explicit(720)))]
    pub device_unique_attestation: Option<()>,
    #[rasn(tag(explicit(723)))]
    pub attestation_id_second_imei: Option<OctetString>,
}

// RootOfTrust ::= SEQUENCE {
//     verifiedBootKey  OCTET_STRING,
//     deviceLocked  BOOLEAN,
//     verifiedBootState  VerifiedBootState,
//     verifiedBootHash OCTET_STRING,
// }
#[derive(Debug, Clone, PartialEq, Eq, AsnType, Decode)]
#[cfg_attr(feature = "encode", derive(rasn::Encode))]
pub struct RootOfTrust {
    pub verified_boot_key: OctetString,
    pub device_locked: bool,
    pub verified_boot_state: VerifiedBootState,
    pub verified_boot_hash: OctetString,
}

// VerifiedBootState ::= ENUMERATED {
//     Verified  (0),
//     SelfSigned  (1),
//     Unverified  (2),
//     Failed  (3),
// }
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsnType, Decode)]
#[cfg_attr(feature = "encode", derive(rasn::Encode))]
#[rasn(enumerated)]
pub enum VerifiedBootState {
    Verified,
    SelfSigned,
    Unverified,
    Failed,
}

#[cfg(feature = "encode")]
mod mock {
    use super::*;

    impl KeyDescription {
        pub fn new_valid_mock(attestation_challenge: Vec<u8>) -> Self {
            KeyDescription {
                attestation_version: 200.into(),
                attestation_security_level: SecurityLevel::TrustedEnvironment,
                key_mint_version: 300.into(),
                key_mint_security_level: SecurityLevel::TrustedEnvironment,
                attestation_challenge: OctetString::from(attestation_challenge),
                unique_id: OctetString::copy_from_slice(b"unique_id"),
                software_enforced: Default::default(),
                hardware_enforced: Default::default(),
            }
        }
    }
}
