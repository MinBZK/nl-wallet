//! This file implements a schema that supports the following versions of the Android Key Attestation Extension schema:
//!
//! - 3
//! - 4
//! - 100
//! - 200
//! - 300

use rasn::types::Integer;
use rasn::types::OctetString;
use rasn::types::SetOf;
use rasn::AsnType;
use rasn::Decode;
use rasn::Decoder;
use rasn::Encode;
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
#[derive(Debug, Clone, PartialEq, Eq, AsnType, Decode, Encode)]
pub(super) struct KeyDescription {
    pub(super) attestation_version: Integer,
    pub(super) attestation_security_level: SecurityLevel,
    pub(super) key_mint_version: Integer,
    pub(super) key_mint_security_level: SecurityLevel,
    pub(super) attestation_challenge: OctetString,
    pub(super) unique_id: OctetString,
    pub(super) software_enforced: AuthorizationList,
    pub(super) hardware_enforced: AuthorizationList,
}

// SecurityLevel ::= ENUMERATED {
//     Software  (0),
//     TrustedEnvironment  (1),
//     StrongBox  (2),
// }
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsnType, Decode, Encode)]
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
#[derive(Debug, Clone, PartialEq, Eq, Default, AsnType, Decode, Encode)]
pub(super) struct AuthorizationList {
    #[rasn(tag(explicit(1)))]
    pub(super) purpose: Option<SetOf<Integer>>,
    #[rasn(tag(explicit(2)))]
    pub(super) algorithm: Option<Integer>,
    #[rasn(tag(explicit(3)))]
    pub(super) key_size: Option<Integer>,
    #[rasn(tag(explicit(5)))]
    pub(super) digest: Option<SetOf<Integer>>,
    #[rasn(tag(explicit(6)))]
    pub(super) padding: Option<SetOf<Integer>>,
    #[rasn(tag(explicit(10)))]
    pub(super) ec_curve: Option<Integer>,
    #[rasn(tag(explicit(200)))]
    pub(super) rsa_public_exponent: Option<Integer>,
    #[rasn(tag(explicit(203)))]
    pub(super) mgf_digest: Option<SetOf<Integer>>,
    #[rasn(tag(explicit(303)))]
    pub(super) rollback_resistance: Option<()>,
    #[rasn(tag(explicit(305)))]
    pub(super) early_boot_only: Option<()>,
    #[rasn(tag(explicit(400)))]
    pub(super) active_date_time: Option<Integer>,
    #[rasn(tag(explicit(401)))]
    pub(super) origination_expire_date_time: Option<Integer>,
    #[rasn(tag(explicit(402)))]
    pub(super) usage_expire_date_time: Option<Integer>,
    #[rasn(tag(explicit(405)))]
    pub(super) usage_count_limit: Option<Integer>,
    #[rasn(tag(explicit(503)))]
    pub(super) no_auth_required: Option<()>,
    #[rasn(tag(explicit(504)))]
    pub(super) user_auth_type: Option<Integer>,
    #[rasn(tag(explicit(505)))]
    pub(super) auth_timeout: Option<Integer>,
    #[rasn(tag(explicit(506)))]
    pub(super) allow_while_on_body: Option<()>,
    #[rasn(tag(explicit(507)))]
    pub(super) trusted_user_presence_required: Option<()>,
    #[rasn(tag(explicit(508)))]
    pub(super) trusted_confirmation_required: Option<()>,
    #[rasn(tag(explicit(509)))]
    pub(super) unlocked_device_required: Option<()>,
    // kept for backwards compatibility reasons, this field is removed in version 100
    #[rasn(tag(explicit(600)))]
    pub(super) all_applications: Option<()>,
    #[rasn(tag(explicit(701)))]
    pub(super) creation_date_time: Option<Integer>,
    #[rasn(tag(explicit(702)))]
    pub(super) origin: Option<Integer>,
    #[rasn(tag(explicit(704)))]
    pub(super) root_of_trust: Option<RootOfTrust>,
    #[rasn(tag(explicit(705)))]
    pub(super) os_version: Option<Integer>,
    #[rasn(tag(explicit(706)))]
    pub(super) os_patch_level: Option<Integer>,
    #[rasn(tag(explicit(709)))]
    pub(super) attestation_application_id: Option<OctetString>,
    #[rasn(tag(explicit(710)))]
    pub(super) attestation_id_brand: Option<OctetString>,
    #[rasn(tag(explicit(711)))]
    pub(super) attestation_id_device: Option<OctetString>,
    #[rasn(tag(explicit(712)))]
    pub(super) attestation_id_product: Option<OctetString>,
    #[rasn(tag(explicit(713)))]
    pub(super) attestation_id_serial: Option<OctetString>,
    #[rasn(tag(explicit(714)))]
    pub(super) attestation_id_imei: Option<OctetString>,
    #[rasn(tag(explicit(715)))]
    pub(super) attestation_id_meid: Option<OctetString>,
    #[rasn(tag(explicit(716)))]
    pub(super) attestation_id_manufacturer: Option<OctetString>,
    #[rasn(tag(explicit(717)))]
    pub(super) attestation_id_model: Option<OctetString>,
    #[rasn(tag(explicit(718)))]
    pub(super) vendor_patch_level: Option<Integer>,
    #[rasn(tag(explicit(719)))]
    pub(super) boot_patch_level: Option<Integer>,
    #[rasn(tag(explicit(720)))]
    pub(super) device_unique_attestation: Option<()>,
    #[rasn(tag(explicit(723)))]
    pub(super) attestation_id_second_imei: Option<OctetString>,
}

// RootOfTrust ::= SEQUENCE {
//     verifiedBootKey  OCTET_STRING,
//     deviceLocked  BOOLEAN,
//     verifiedBootState  VerifiedBootState,
//     verifiedBootHash OCTET_STRING,
// }
#[derive(Debug, Clone, PartialEq, Eq, AsnType, Decode, Encode)]
pub struct RootOfTrust {
    pub(super) verified_boot_key: OctetString,
    pub(super) device_locked: bool,
    pub(super) verified_boot_state: VerifiedBootState,
    pub(super) verified_boot_hash: OctetString,
}

// VerifiedBootState ::= ENUMERATED {
//     Verified  (0),
//     SelfSigned  (1),
//     Unverified  (2),
//     Failed  (3),
// }
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsnType, Decode, Encode)]
#[rasn(enumerated)]
pub enum VerifiedBootState {
    Verified,
    SelfSigned,
    Unverified,
    Failed,
}
