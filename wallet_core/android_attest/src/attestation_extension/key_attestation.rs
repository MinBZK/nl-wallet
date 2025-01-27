//! The types in this file are based on the types in
//!
//! - Android documentation: https://source.android.com/docs/security/features/keystore/tags
//! - KeyMint source code: https://android.googlesource.com/platform/system/keymint/
//!
//! These types should be able to handle all KeyMaster/KeyMint versions, so for example enums
//! might contain more values than defined in the latest KeyMint version.

use std::collections::HashSet;
use std::hash::Hash;
use std::time::Duration;

use bitflags::bitflags;
use chrono::DateTime;
use chrono::Utc;
use int_enum::IntEnum;
use rasn::error::DecodeError;
use rasn::types::Integer;
use rasn::types::OctetString;
use rasn::types::SetOf;
use rasn::AsnType;
use rasn::Decode;
use rasn::Decoder;

use super::key_description;
use super::key_description::KeyDescription;
use super::key_description::RootOfTrust;
use super::key_description::SecurityLevel;

macro_rules! integer_int_enum_conversion {
    ($type:ty, $repr:ty, $error_type:ident, $invalid_error:ident) => {
        #[derive(Debug, thiserror::Error)]
        #[cfg_attr(test, derive(PartialEq, Eq))]
        pub enum $error_type {
            #[error("could not convert Integer to {}: {0}", stringify!($repr))]
            IntegerConversion(Integer),
            #[error("not a valid {}: {0}", stringify!($type))]
            $invalid_error($repr),
        }
        impl TryFrom<&Integer> for $type {
            type Error = $error_type;
            fn try_from(value: &Integer) -> Result<Self, Self::Error> {
                let repr: $repr = value
                    .try_into()
                    .map_err(|_| $error_type::IntegerConversion(value.clone()))?;
                <$type>::try_from(repr).map_err($error_type::$invalid_error)
            }
        }
    };
}

macro_rules! integer_int_enum_conversion_with_set {
    ($type:ty, $repr:ident, $error_type:ident, $invalid_error:ident) => {
        integer_int_enum_conversion!($type, $repr, $error_type, $invalid_error);
        impl $type {
            fn from_set_of_integer(set: SetOf<Integer>) -> Result<HashSet<$type>, $error_type> {
                set.to_vec()
                    .into_iter()
                    .map(|purpose| TryFrom::try_from(purpose))
                    .collect::<Result<HashSet<_>, _>>()
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, IntEnum)]
#[repr(i32)]
pub enum AttestationVersion {
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V100 = 100,
    V200 = 200,
    V300 = 300,
}

integer_int_enum_conversion!(
    AttestationVersion,
    i32,
    AttestationVersionError,
    InvalidAttestationVersion
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, IntEnum)]
#[repr(i32)]
pub enum KeyMintVersion {
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V41 = 41,
    V100 = 100,
    V200 = 200,
    V300 = 300,
}

integer_int_enum_conversion!(KeyMintVersion, i32, KeyMintVersionError, InvalidKeyMintVersion);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, IntEnum)]
#[repr(u32)]
pub enum KeyPurpose {
    Encrypt = 0,
    Decrypt = 1,
    Sign = 2,
    Verify = 3,
    DeriveKey = 4, // Used in KeyMaster, removed in KeyMint
    WrapKey = 5,
    AgreeKey = 6,
    AttestKey = 7,
}

integer_int_enum_conversion_with_set!(KeyPurpose, u32, KeyPurposeError, InvalidKeyPurpose);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, IntEnum)]
#[repr(u32)]
pub enum Algorithm {
    Rsa = 1,
    Ec = 3,
    Aes = 32,
    TrippleDes = 33,
    Hmac = 128,
}

integer_int_enum_conversion!(Algorithm, u32, AlgorithmError, InvalidAlgorithm);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, IntEnum)]
#[repr(u32)]
pub enum Digest {
    None = 0,
    Md5 = 1,
    Sha1 = 2,
    Sha2_224 = 3,
    Sha2_256 = 4,
    Sha2_384 = 5,
    Sha2_512 = 6,
}

integer_int_enum_conversion_with_set!(Digest, u32, DigestError, InvalidDigest);

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, IntEnum)]
#[repr(u32)]
pub enum Padding {
    None = 1,
    RsaOaep = 2,
    RsaPss = 3,
    RsaPkcs1_1_5_Encrypt = 4,
    RsaPkcs1_1_5_Sign = 5,
    Pkcs7 = 64,
}

integer_int_enum_conversion_with_set!(Padding, u32, PaddingError, InvalidPadding);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, IntEnum)]
#[repr(u32)]
pub enum EcCurve {
    P224 = 0,
    P256 = 1,
    P384 = 2,
    P512 = 3,
    Curve25519 = 4,
}

integer_int_enum_conversion!(EcCurve, u32, EcCurveError, InvalidEcCurve);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareAuthenticatorType(u32);

impl TryFrom<Integer> for HardwareAuthenticatorType {
    type Error = Integer;
    fn try_from(value: Integer) -> Result<Self, Self::Error> {
        let val: u32 = (&value).try_into().map_err(|_| value)?;
        Ok(Self(val))
    }
}

bitflags! {
    impl HardwareAuthenticatorType: u32 {
        const PASSWORD = 1 << 0;
        const FINGERPRINT = 1 << 1;

        // The source may set any bits
        const _ = !0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, IntEnum)]
#[repr(u32)]
pub enum KeyOrigin {
    Generated = 0,
    Derived = 1,
    Imported = 2,
    Unknown = 3,
    SecurelyImported = 4,
}

integer_int_enum_conversion!(KeyOrigin, u32, KeyOriginError, InvalidKeyOrigin);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OsVersion {
    pub major: u8,
    pub minor: u8,
    pub sub_minor: u8,
}

impl OsVersion {
    pub fn new(major: u8, minor: u8, sub_minor: u8) -> Self {
        Self {
            major,
            minor,
            sub_minor,
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum OsVersionError {
    #[error("could not convert Integer to usize: {0}")]
    IntegerConversion(Integer),
    #[error("not a valid OsVersion: ")]
    InvalidOsVersion(usize),
}

impl TryFrom<Integer> for OsVersion {
    type Error = OsVersionError;

    fn try_from(value: Integer) -> Result<Self, Self::Error> {
        let version: usize = (&value)
            .try_into()
            .map_err(|_| OsVersionError::IntegerConversion(value))?;
        let major = version / 10_000;
        if major >= 100 {
            return Err(OsVersionError::InvalidOsVersion(version));
        }
        let minor = version / 100 % 100;
        let sub_minor = version % 100;
        // unwraps are safe because of guards above
        Ok(OsVersion {
            major: major.try_into().unwrap(),
            minor: minor.try_into().unwrap(),
            sub_minor: sub_minor.try_into().unwrap(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DateTimeError {
    #[error("could not convert Integer to i64: {0}")]
    IntegerConversion(Integer),
    #[error("not a valid DateTime: {0}")]
    InvalidDateTime(i64),
}

fn date_time_from_integer_milliseconds(source: Integer) -> Result<DateTime<Utc>, DateTimeError> {
    let millis: i64 = (&source)
        .try_into()
        .map_err(|_| DateTimeError::IntegerConversion(source))?;
    DateTime::from_timestamp_millis(millis).ok_or(DateTimeError::InvalidDateTime(millis))
}

fn duration_from_seconds(source: Integer) -> Result<Duration, Integer> {
    let seconds: u64 = (&source).try_into().map_err(|_| source)?;
    Ok(Duration::from_secs(seconds))
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum PatchLevelError {
    #[error("conversion error from: {0}")]
    Conversion(Integer),
    #[error("invalid date: {0}")]
    InvalidDate(Integer),
}

/// Decoded patch_level.
/// Supports both YYYYMM and YYYYMMDD notations, including values 0 or 00 for MM and/or DD.
///
/// - os_patch_level: YYYYMM
/// - vendor_patch_level: YYYYMMDD, but also found YYYYMM
/// - boot_patch_level: YYYYMMDD, but also found YYYYMM
///
/// Exceptional cases:
/// It is not possible to use a `Date` type for `PatchLevel`, because of the following finds
/// in test data derived from Google source repositories and Android emulators.
///
/// - Sometimes DD is set to `00`, which is not a valid date
/// - Sometimes the whole `PatchLevel` was set to `0`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PatchLevel {
    year: u16,
    month: u8,
    day: Option<u8>,
}

impl PatchLevel {
    pub fn new(year: u16, month: u8, day: Option<u8>) -> Self {
        Self { year, month, day }
    }
}

impl TryFrom<Integer> for PatchLevel {
    type Error = PatchLevelError;

    fn try_from(value: Integer) -> Result<Self, Self::Error> {
        let mut rest: usize = (&value)
            .try_into()
            .map_err(|_| PatchLevelError::Conversion(value.clone()))?;

        if rest == 0 {
            return Ok(PatchLevel::new(0, 0, None));
        } else if rest < 10_000 {
            return Err(PatchLevelError::InvalidDate(value));
        }

        let first = rest % 100;
        rest /= 100;

        let result = if rest < 10_000 {
            PatchLevel::new(rest as u16, first as u8, None)
        } else {
            let second = rest % 100;
            rest /= 100;
            if rest > 10_000 {
                return Err(PatchLevelError::InvalidDate(value));
            }
            PatchLevel::new(rest as u16, second as u8, Some(first as u8))
        };

        Ok(result)
    }
}

// AttestationApplicationId ::= SEQUENCE {
//     package_infos  SET OF AttestationPackageInfo,
//     signature_digests  SET OF OCTET_STRING,
// }
#[derive(Debug, Clone, PartialEq, Eq, AsnType, Decode)]
#[cfg_attr(feature = "encode", derive(rasn::Encode))]
pub struct AttestationApplicationId {
    pub package_infos: SetOf<AttestationPackageInfo>,
    pub signature_digests: SetOf<OctetString>,
}

// AttestationPackageInfo ::= SEQUENCE {
//     package_name  OCTET_STRING,
//     version  INTEGER,
// }
#[derive(Debug, Clone, PartialEq, Eq, Hash, AsnType, Decode)]
#[cfg_attr(feature = "encode", derive(rasn::Encode))]
pub struct AttestationPackageInfo {
    pub package_name: OctetString,
    pub version: Integer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyAttestation {
    pub attestation_version: AttestationVersion,
    pub attestation_security_level: SecurityLevel,
    pub key_mint_version: KeyMintVersion,
    pub key_mint_security_level: SecurityLevel,
    pub attestation_challenge: OctetString,
    pub unique_id: OctetString,
    pub software_enforced: AuthorizationList,
    pub hardware_enforced: AuthorizationList,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum KeyAttestationVerificationError {
    #[error("attestation challenge mismatch")]
    AttestationChallenge,
    #[error("security requirements not met for attestation_security_level: {0:?}")]
    AttestationSecurityLevel(SecurityLevel),
    #[error("security requirements not met for key_mint_security_level: {0:?}")]
    KeyMintSecurityLevel(SecurityLevel),
}

impl KeyAttestation {
    pub fn verify(&self, attestation_challenge: &[u8]) -> Result<(), KeyAttestationVerificationError> {
        if self.attestation_challenge != attestation_challenge {
            return Err(KeyAttestationVerificationError::AttestationChallenge);
        }

        self.attestation_security_level
            .verify()
            .map_err(KeyAttestationVerificationError::AttestationSecurityLevel)?;
        self.key_mint_security_level
            .verify()
            .map_err(KeyAttestationVerificationError::KeyMintSecurityLevel)?;

        Ok(())
    }
}

impl SecurityLevel {
    pub fn verify(&self) -> Result<(), SecurityLevel> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "allow_emulator_keys")] {
                // Allow any security level on the emulator.
                tracing::debug!("Allowing all security levels on android emulator");
            } else {
                if !match self {
                    SecurityLevel::Software => false,
                    SecurityLevel::TrustedEnvironment | SecurityLevel::StrongBox => true,
                } {
                    return Err(*self);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeyDescriptionFieldError {
    #[error("invalid attestation_version field: {0}")]
    AttestationVersion(#[from] AttestationVersionError),
    #[error("invalid key_mint_version field: {0}")]
    KeyMintVersion(#[from] KeyMintVersionError),
    #[error("invalid software_enforced field: {0}")]
    SoftwareEnforced(#[source] AuthorizationListFieldError),
    #[error("invalid hardware_enforced field: {0}")]
    HardwareEnforced(#[source] AuthorizationListFieldError),
}

impl TryFrom<KeyDescription> for KeyAttestation {
    type Error = KeyDescriptionFieldError;

    fn try_from(source: KeyDescription) -> Result<Self, Self::Error> {
        let result = KeyAttestation {
            attestation_version: (&source.attestation_version).try_into()?,
            attestation_security_level: source.attestation_security_level,
            key_mint_version: (&source.key_mint_version).try_into()?,
            key_mint_security_level: source.key_mint_security_level,
            attestation_challenge: source.attestation_challenge,
            unique_id: source.unique_id,
            software_enforced: source
                .software_enforced
                .try_into()
                .map_err(KeyDescriptionFieldError::SoftwareEnforced)?,
            hardware_enforced: source
                .hardware_enforced
                .try_into()
                .map_err(KeyDescriptionFieldError::HardwareEnforced)?,
        };

        Ok(result)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthorizationList {
    pub purpose: Option<HashSet<KeyPurpose>>,
    pub algorithm: Option<Algorithm>,
    pub key_size: Option<u32>,
    pub digest: Option<HashSet<Digest>>,
    pub padding: Option<HashSet<Padding>>,
    pub ec_curve: Option<EcCurve>,
    pub rsa_public_exponent: Option<u64>,
    pub mgf_digest: Option<HashSet<Digest>>,
    pub rollback_resistance: bool,
    pub early_boot_only: bool,
    pub active_date_time: Option<DateTime<Utc>>, // milliseconds since January 1, 1970
    pub origination_expire_date_time: Option<DateTime<Utc>>, // milliseconds since January 1, 1970
    pub usage_expire_date_time: Option<DateTime<Utc>>, // milliseconds since January 1, 1970
    pub usage_count_limit: Option<u32>,
    pub no_auth_required: bool,
    pub user_auth_type: Option<HardwareAuthenticatorType>,
    pub auth_timeout: Option<Duration>, // in seconds
    pub allow_while_on_body: bool,
    pub trusted_user_presence_required: bool,
    pub trusted_confirmation_required: bool,
    pub unlocked_device_required: bool,
    pub all_applications: bool,
    pub creation_date_time: Option<DateTime<Utc>>,
    pub origin: Option<KeyOrigin>,
    pub root_of_trust: Option<RootOfTrust>,
    pub os_version: Option<OsVersion>,
    pub os_patch_level: Option<PatchLevel>,
    pub attestation_application_id: Option<AttestationApplicationId>,
    pub attestation_id_brand: Option<OctetString>,
    pub attestation_id_device: Option<OctetString>,
    pub attestation_id_product: Option<OctetString>,
    pub attestation_id_serial: Option<OctetString>,
    pub attestation_id_imei: Option<OctetString>,
    pub attestation_id_meid: Option<OctetString>,
    pub attestation_id_manufacturer: Option<OctetString>,
    pub attestation_id_model: Option<OctetString>,
    pub vendor_patch_level: Option<PatchLevel>,
    pub boot_patch_level: Option<PatchLevel>,
    pub device_unique_attestation: bool,
    pub attestation_id_second_imei: Option<OctetString>,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthorizationListFieldError {
    #[error("invalid purpose field: {0}")]
    Purpose(#[from] KeyPurposeError),
    #[error("invalid algorithm field: {0}")]
    Algorithm(#[from] AlgorithmError),
    #[error("invalid padding field: {0}")]
    Padding(#[from] PaddingError),
    #[error("invalid digest field: {0}")]
    Digest(#[from] DigestError),
    #[error("invalid ec_curve field: {0}")]
    EcCurve(#[from] EcCurveError),
    #[error("invalid rsa_public_exponent field: {0}")]
    RsaPublicExponent(Integer),
    #[error("invalid active_date_time field: {0}")]
    ActiveDateTime(DateTimeError),
    #[error("invalid origination_expire_date_time field: {0}")]
    OriginationExpireDateTime(DateTimeError),
    #[error("invalid usage_expire_date_time field: {0}")]
    UsageExpireDateTime(DateTimeError),
    #[error("invalid user_auth_type field: {0}")]
    UserAuthType(Integer),
    #[error("invalid auth_timeout field: {0}")]
    AuthTimeout(Integer),
    #[error("invalid creation_date_time field: {0}")]
    CreationDateTime(DateTimeError),
    #[error("invalid origin field: {0}")]
    Origin(#[from] KeyOriginError),
    #[error("invalid os_version field: {0}")]
    OsVersion(#[from] OsVersionError),
    #[error("invalid os_patch_level field: {0}")]
    OsPatchLevel(#[source] PatchLevelError),
    #[error("invalid vendor_patch_level field: {0}")]
    VendorPatchLevel(#[source] PatchLevelError),
    #[error("invalid boot_patch_level field: {0}")]
    BootPatchLevel(#[source] PatchLevelError),
    #[error("invalid attestation_application_id field: {0}")]
    AttestationApplicationId(#[source] DecodeError),
    #[error("invalid usage_count_limit field: {0}")]
    UsageCountLimit(Integer),
    #[error("invalid key_size field: {0}")]
    KeySize(Integer),
}

impl TryFrom<key_description::AuthorizationList> for AuthorizationList {
    type Error = AuthorizationListFieldError;

    fn try_from(source: key_description::AuthorizationList) -> Result<Self, Self::Error> {
        let result = AuthorizationList {
            purpose: source.purpose.map(KeyPurpose::from_set_of_integer).transpose()?,
            algorithm: source.algorithm.as_ref().map(TryFrom::try_from).transpose()?,
            key_size: source
                .key_size
                .as_ref()
                .map(TryFrom::try_from)
                .transpose()
                .map_err(|_| AuthorizationListFieldError::KeySize(source.key_size.unwrap()))?,
            digest: source.digest.map(Digest::from_set_of_integer).transpose()?,
            padding: source.padding.map(Padding::from_set_of_integer).transpose()?,
            ec_curve: source.ec_curve.as_ref().map(TryFrom::try_from).transpose()?,
            rsa_public_exponent: source
                .rsa_public_exponent
                .as_ref()
                .map(TryFrom::try_from)
                .transpose()
                .map_err(|_| AuthorizationListFieldError::RsaPublicExponent(source.rsa_public_exponent.unwrap()))?,
            mgf_digest: source.mgf_digest.map(Digest::from_set_of_integer).transpose()?,
            rollback_resistance: source.rollback_resistance.is_some(),
            early_boot_only: source.early_boot_only.is_some(),
            active_date_time: source
                .active_date_time
                .map(date_time_from_integer_milliseconds)
                .transpose()
                .map_err(AuthorizationListFieldError::ActiveDateTime)?,
            origination_expire_date_time: source
                .origination_expire_date_time
                .map(date_time_from_integer_milliseconds)
                .transpose()
                .map_err(AuthorizationListFieldError::OriginationExpireDateTime)?,
            usage_expire_date_time: source
                .usage_expire_date_time
                .map(date_time_from_integer_milliseconds)
                .transpose()
                .map_err(AuthorizationListFieldError::UsageExpireDateTime)?,
            usage_count_limit: source
                .usage_count_limit
                .as_ref()
                .map(TryFrom::try_from)
                .transpose()
                .map_err(|_| AuthorizationListFieldError::UsageCountLimit(source.usage_count_limit.unwrap()))?,
            no_auth_required: source.no_auth_required.is_some(),
            user_auth_type: source
                .user_auth_type
                .map(TryFrom::try_from)
                .transpose()
                .map_err(AuthorizationListFieldError::UserAuthType)?,
            auth_timeout: source
                .auth_timeout
                .map(duration_from_seconds)
                .transpose()
                .map_err(AuthorizationListFieldError::AuthTimeout)?,
            allow_while_on_body: source.allow_while_on_body.is_some(),
            trusted_user_presence_required: source.trusted_user_presence_required.is_some(),
            trusted_confirmation_required: source.trusted_confirmation_required.is_some(),
            unlocked_device_required: source.unlocked_device_required.is_some(),
            all_applications: source.all_applications.is_some(),
            creation_date_time: source
                .creation_date_time
                .map(date_time_from_integer_milliseconds)
                .transpose()
                .map_err(AuthorizationListFieldError::CreationDateTime)?,
            origin: source.origin.as_ref().map(TryFrom::try_from).transpose()?,
            root_of_trust: source.root_of_trust,
            os_version: source.os_version.map(TryFrom::try_from).transpose()?,
            os_patch_level: source
                .os_patch_level
                .map(TryFrom::try_from)
                .transpose()
                .map_err(AuthorizationListFieldError::OsPatchLevel)?,
            attestation_application_id: source
                .attestation_application_id
                .map(|bytes| rasn::der::decode(&bytes))
                .transpose()
                .map_err(AuthorizationListFieldError::AttestationApplicationId)?,
            attestation_id_brand: source.attestation_id_brand,
            attestation_id_device: source.attestation_id_device,
            attestation_id_product: source.attestation_id_product,
            attestation_id_serial: source.attestation_id_serial,
            attestation_id_imei: source.attestation_id_imei,
            attestation_id_meid: source.attestation_id_meid,
            attestation_id_manufacturer: source.attestation_id_manufacturer,
            attestation_id_model: source.attestation_id_model,
            vendor_patch_level: source
                .vendor_patch_level
                .map(TryFrom::try_from)
                .transpose()
                .map_err(AuthorizationListFieldError::VendorPatchLevel)?,
            boot_patch_level: source
                .boot_patch_level
                .map(TryFrom::try_from)
                .transpose()
                .map_err(AuthorizationListFieldError::BootPatchLevel)?,
            device_unique_attestation: source.device_unique_attestation.is_some(),
            attestation_id_second_imei: source.attestation_id_second_imei,
        };

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use chrono::NaiveDate;
    use rstest::rstest;

    use super::key_description::VerifiedBootState;

    use super::*;

    #[rstest]
    #[case(1.into(), Ok(AttestationVersion::V1))]
    #[case(2.into(), Ok(AttestationVersion::V2))]
    #[case(3.into(), Ok(AttestationVersion::V3))]
    #[case(4.into(), Ok(AttestationVersion::V4))]
    #[case(100.into(), Ok(AttestationVersion::V100))]
    #[case(200.into(), Ok(AttestationVersion::V200))]
    #[case(300.into(), Ok(AttestationVersion::V300))]
    #[case(0.into(), Err(AttestationVersionError::InvalidAttestationVersion(0)))]
    #[case(5.into(), Err(AttestationVersionError::InvalidAttestationVersion(5)))]
    #[case(
        400.into(),
        Err(AttestationVersionError::InvalidAttestationVersion(400))
    )]
    fn attestation_version(
        #[case] input: Integer,
        #[case] expected: Result<AttestationVersion, AttestationVersionError>,
    ) {
        assert_eq!(AttestationVersion::try_from(&input), expected);
    }

    #[rstest]
    #[case(2.into(), Ok(KeyMintVersion::V2))]
    #[case(3.into(), Ok(KeyMintVersion::V3))]
    #[case(4.into(), Ok(KeyMintVersion::V4))]
    #[case(41.into(), Ok(KeyMintVersion::V41))]
    #[case(100.into(), Ok(KeyMintVersion::V100))]
    #[case(200.into(), Ok(KeyMintVersion::V200))]
    #[case(300.into(), Ok(KeyMintVersion::V300))]
    #[case(0.into(), Err(KeyMintVersionError::InvalidKeyMintVersion(0)))]
    #[case(1.into(), Err(KeyMintVersionError::InvalidKeyMintVersion(1)))]
    #[case(5.into(), Err(KeyMintVersionError::InvalidKeyMintVersion(5)))]
    #[case(400.into(), Err(KeyMintVersionError::InvalidKeyMintVersion(400)))]
    fn key_mint_version(#[case] input: Integer, #[case] expected: Result<KeyMintVersion, KeyMintVersionError>) {
        assert_eq!(KeyMintVersion::try_from(&input), expected);
    }

    #[rstest]
    #[case(0.into(), Ok(KeyPurpose::Encrypt))]
    #[case(1.into(), Ok(KeyPurpose::Decrypt))]
    #[case(2.into(), Ok(KeyPurpose::Sign))]
    #[case(3.into(), Ok(KeyPurpose::Verify))]
    #[case(4.into(), Ok(KeyPurpose::DeriveKey))]
    #[case(5.into(), Ok(KeyPurpose::WrapKey))]
    #[case(6.into(), Ok(KeyPurpose::AgreeKey))]
    #[case(7.into(), Ok(KeyPurpose::AttestKey))]
    #[case(8.into(), Err(KeyPurposeError::InvalidKeyPurpose(8)))]
    fn key_purpose(#[case] input: Integer, #[case] expected: Result<KeyPurpose, KeyPurposeError>) {
        assert_eq!(KeyPurpose::try_from(&input), expected);
    }

    #[rstest]
    #[case(1.into(), Ok(Algorithm::Rsa))]
    #[case(3.into(), Ok(Algorithm::Ec))]
    #[case(32.into(), Ok(Algorithm::Aes))]
    #[case(33.into(), Ok(Algorithm::TrippleDes))]
    #[case(128.into(), Ok(Algorithm::Hmac))]
    #[case(0.into(), Err(AlgorithmError::InvalidAlgorithm(0)))]
    #[case(2.into(), Err(AlgorithmError::InvalidAlgorithm(2)))]
    fn algorithm(#[case] input: Integer, #[case] expected: Result<Algorithm, AlgorithmError>) {
        assert_eq!(Algorithm::try_from(&input), expected);
    }

    #[rstest]
    #[case(0.into(), Ok(Digest::None))]
    #[case(1.into(), Ok(Digest::Md5))]
    #[case(2.into(), Ok(Digest::Sha1))]
    #[case(3.into(), Ok(Digest::Sha2_224))]
    #[case(4.into(), Ok(Digest::Sha2_256))]
    #[case(5.into(), Ok(Digest::Sha2_384))]
    #[case(6.into(), Ok(Digest::Sha2_512))]
    #[case(7.into(), Err(DigestError::InvalidDigest(7)))]
    fn digest(#[case] input: Integer, #[case] expected: Result<Digest, DigestError>) {
        assert_eq!(Digest::try_from(&input), expected);
    }

    #[rstest]
    #[case(1.into(), Ok(Padding::None))]
    #[case(2.into(), Ok(Padding::RsaOaep))]
    #[case(3.into(), Ok(Padding::RsaPss))]
    #[case(4.into(), Ok(Padding::RsaPkcs1_1_5_Encrypt))]
    #[case(5.into(), Ok(Padding::RsaPkcs1_1_5_Sign))]
    #[case(64.into(), Ok(Padding::Pkcs7))]
    #[case(0.into(), Err(PaddingError::InvalidPadding(0)))]
    #[case(6.into(), Err(PaddingError::InvalidPadding(6)))]
    fn padding(#[case] input: Integer, #[case] expected: Result<Padding, PaddingError>) {
        assert_eq!(Padding::try_from(&input), expected);
    }

    #[rstest]
    #[case(0.into(), Ok(EcCurve::P224))]
    #[case(1.into(), Ok(EcCurve::P256))]
    #[case(2.into(), Ok(EcCurve::P384))]
    #[case(3.into(), Ok(EcCurve::P512))]
    #[case(4.into(), Ok(EcCurve::Curve25519))]
    #[case(5.into(), Err(EcCurveError::InvalidEcCurve(5)))]
    fn ec_curve(#[case] input: Integer, #[case] expected: Result<EcCurve, EcCurveError>) {
        assert_eq!(EcCurve::try_from(&input), expected);
    }

    #[rstest]
    #[case(0.into(), Ok(KeyOrigin::Generated))]
    #[case(1.into(), Ok(KeyOrigin::Derived))]
    #[case(2.into(), Ok(KeyOrigin::Imported))]
    #[case(3.into(), Ok(KeyOrigin::Unknown))]
    #[case(4.into(), Ok(KeyOrigin::SecurelyImported))]
    #[case(5.into(), Err(KeyOriginError::InvalidKeyOrigin(5)))]
    fn key_origin(#[case] input: Integer, #[case] expected: Result<KeyOrigin, KeyOriginError>) {
        assert_eq!(KeyOrigin::try_from(&input), expected);
    }

    #[rstest]
    #[case(40_003.into(), Ok((4, 0, 3)))]
    #[case(999_999.into(), Ok((99, 99, 99)))]
    #[case(
        1_000_000.into(),
        Err(OsVersionError::InvalidOsVersion(1_000_000))
    )]
    #[case(
        4_040_003.into(),
        Err(OsVersionError::InvalidOsVersion(4_040_003))
    )]
    fn os_version(#[case] input: Integer, #[case] expected: Result<(u8, u8, u8), OsVersionError>) {
        let actual = OsVersion::try_from(input);
        assert_eq!(actual.is_ok(), expected.is_ok());
        match (actual, expected) {
            (Err(e1), Err(e2)) => assert_eq!(e1, e2),
            (Ok(version), Ok((major, minor, bugfix))) => {
                assert_eq!(version.major, major);
                assert_eq!(version.minor, minor);
                assert_eq!(version.sub_minor, bugfix);
            }
            _ => unreachable!(),
        }
    }

    #[rstest]
    #[case(OsVersion::new(1, 2, 3), OsVersion::new(2, 0, 0), Ordering::Less)]
    #[case(OsVersion::new(1, 2, 3), OsVersion::new(1, 0, 0), Ordering::Greater)]
    #[case(OsVersion::new(1, 2, 3), OsVersion::new(1, 2, 2), Ordering::Greater)]
    #[case(OsVersion::new(1, 2, 3), OsVersion::new(1, 2, 3), Ordering::Equal)]
    fn os_version_ord(#[case] first: OsVersion, #[case] second: OsVersion, #[case] expected: Ordering) {
        assert_eq!(first.cmp(&second), expected);
    }

    #[rstest]
    #[case(Integer::ZERO, Ok(PatchLevel::new(0, 0, None)))]
    #[case(
        2019.into(),
        Err(PatchLevelError::InvalidDate(2019.into()))
    )]
    #[case(201907.into(), Ok(PatchLevel::new(2019, 7, None)))]
    #[case(201913.into(), Ok(PatchLevel::new(2019, 13, None)))]
    #[case(20190705.into(), Ok(PatchLevel::new(2019, 7, Some(5))))]
    #[case(20191305.into(), Ok(PatchLevel::new(2019, 13, Some(5))))]
    #[case(20190732.into(), Ok(PatchLevel::new(2019, 7, Some(32))))]
    #[case(20190229.into(), Ok(PatchLevel::new(2019, 2, Some(29))))]
    #[case(
        120190705.into(),
        Err(PatchLevelError::InvalidDate(120190705.into()))
    )]
    fn patch_level(#[case] input: Integer, #[case] expected: Result<PatchLevel, PatchLevelError>) {
        let actual: Result<PatchLevel, PatchLevelError> = input.try_into();
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case(0.into(), &[])]
    #[case(1.into(), &[HardwareAuthenticatorType::PASSWORD])]
    #[case(2.into(), &[HardwareAuthenticatorType::FINGERPRINT])]
    #[case(3.into(), &[HardwareAuthenticatorType::PASSWORD, HardwareAuthenticatorType::FINGERPRINT])]
    #[case(Integer::Primitive(u32::MAX as isize), &[HardwareAuthenticatorType::PASSWORD, HardwareAuthenticatorType::FINGERPRINT])]
    fn hardware_authenticator_type(#[case] input: Integer, #[case] expected: &[HardwareAuthenticatorType]) {
        let actual = HardwareAuthenticatorType::try_from(input.clone()).unwrap();

        for expect in expected.iter() {
            assert!(actual.contains(expect.clone()));
        }
        if expected.is_empty() {
            assert!(actual.is_empty());
        }
        if input == Integer::Primitive(u32::MAX as isize) {
            assert!(actual.is_all());
        }
    }

    #[rstest]
    #[case(1735035371011_u128.into(), NaiveDate::from_ymd_opt(2024, 12, 24).unwrap().and_hms_milli_opt(10, 16, 11, 11).unwrap().and_utc())]
    #[case(1561115488586_u128.into(), NaiveDate::from_ymd_opt(2019, 6, 21).unwrap().and_hms_milli_opt(11, 11, 28, 586).unwrap().and_utc())]
    #[case(1531381425477_u128.into(), NaiveDate::from_ymd_opt(2018, 7, 12).unwrap().and_hms_milli_opt(7, 43, 45, 477).unwrap().and_utc())]
    fn test_date_time_from_integer_milliseconds(#[case] input: Integer, #[case] expected: DateTime<Utc>) {
        let actual = date_time_from_integer_milliseconds(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case(300.into(), Duration::from_secs(300))]
    fn test_duration_from_seconds(#[case] input: Integer, #[case] expected: Duration) {
        let actual = duration_from_seconds(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    #[allow(clippy::octal_escapes)]
    fn convert_key_description() {
        let input = KeyDescription {
            attestation_version: 200.into(),
            attestation_security_level: SecurityLevel::Software,
            key_mint_version: 200
                .into(),
            key_mint_security_level: SecurityLevel::Software,
            attestation_challenge: OctetString::copy_from_slice(&[116, 104, 105, 115, 95, 105, 115, 95, 97, 95, 99, 104, 97, 108, 108, 101, 110, 103, 101, 95, 115, 116, 114, 105, 110, 103]),
            unique_id: OctetString::new(),
            software_enforced: super::key_description::AuthorizationList {
                purpose: SetOf::from_vec(vec![2.into()]).into(),
                algorithm: Integer::from(3).into(),
                key_size: Integer::from(256).into(),
                digest: SetOf::from_vec(vec![4.into()]).into(),
                padding: SetOf::from_vec(vec![1.into()]).into(),
                ec_curve: Integer::from(1).into(),
                rsa_public_exponent: Integer::from(7).into(),
                mgf_digest: SetOf::from_vec(vec![4.into()]).into(),
                rollback_resistance: ().into(),
                early_boot_only: ().into(),
                active_date_time: Integer::from(1735035371011_i128).into(),
                origination_expire_date_time: Integer::from(1735035371011_i128).into(),
                usage_expire_date_time: Integer::from(1735035371011_i128).into(),
                usage_count_limit: Integer::from(3).into(),
                no_auth_required: ().into(),
                user_auth_type: Integer::from(3).into(),
                auth_timeout: Integer::from(24 * 60 * 60).into(),
                allow_while_on_body: ().into(),
                trusted_user_presence_required: ().into(),
                trusted_confirmation_required: ().into(),
                unlocked_device_required: ().into(),
                all_applications: ().into(),
                creation_date_time: Integer::from(1735035371011_i128).into(),
                origin: Integer::from(0).into(),
                root_of_trust: RootOfTrust {
                    verified_boot_key: OctetString::copy_from_slice(b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"),
                    device_locked: false,
                    verified_boot_state: VerifiedBootState::Unverified,
                    verified_boot_hash: OctetString::copy_from_slice(b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"),
                }.into(),
                os_version: Integer::from(130_000).into(),
                os_patch_level: Integer::from(202403).into(),
                attestation_application_id: OctetString::copy_from_slice(b"0^1806\x041nl.rijksoverheid.edi.wallet.platform_support.test\x02\x01\01\"\x04 \xd3\xa5O\x11T\xc2ZZ\xb3\xf1%(\xdc\xc3r.\x0b\x8e\n\xd8\x11\xd42T\x84\xb7\xb2+\x0e\x8a\x1f\xe3").into(),
                attestation_id_brand: OctetString::copy_from_slice(b"attestation_id_brand").into(),
                attestation_id_device: OctetString::copy_from_slice(b"attestation_id_device").into(),
                attestation_id_product: OctetString::copy_from_slice(b"attestation_id_product").into(),
                attestation_id_serial: OctetString::copy_from_slice(b"attestation_id_serial").into(),
                attestation_id_imei: OctetString::copy_from_slice(b"attestation_id_imei").into(),
                attestation_id_meid: OctetString::copy_from_slice(b"attestation_id_meid").into(),
                attestation_id_manufacturer: OctetString::copy_from_slice(b"attestation_id_manufacturer").into(),
                attestation_id_model: OctetString::copy_from_slice(b"attestation_id_model").into(),
                vendor_patch_level: Integer::from(0).into(),
                boot_patch_level: Integer::from(20240301).into(),
                device_unique_attestation: ().into(),
                attestation_id_second_imei: OctetString::copy_from_slice(b"attestation_id_second_imei").into(),
            },
            hardware_enforced: super::key_description::AuthorizationList {
                ..Default::default()
            },
        };
        let actual = KeyAttestation::try_from(input).expect("test case is valid");

        let expected = KeyAttestation {
            attestation_version: AttestationVersion::V200,
            attestation_security_level: SecurityLevel::Software,
            key_mint_version: KeyMintVersion::V200,
            key_mint_security_level: SecurityLevel::Software,
            attestation_challenge: OctetString::copy_from_slice(b"this_is_a_challenge_string"),
            unique_id: OctetString::copy_from_slice(b""),
            software_enforced: AuthorizationList {
                purpose: Some(vec![KeyPurpose::Sign].into_iter().collect()),
                algorithm: Some(Algorithm::Ec),
                key_size: Some(256),
                digest: Some(vec![Digest::Sha2_256].into_iter().collect()),
                padding: Some(vec![Padding::None].into_iter().collect()),
                ec_curve: Some(EcCurve::P256),
                rsa_public_exponent: Some(7),
                mgf_digest: Some(vec![Digest::Sha2_256].into_iter().collect()),
                rollback_resistance: true,
                early_boot_only: true,
                active_date_time: Some(
                    NaiveDate::from_ymd_opt(2024, 12, 24)
                        .unwrap()
                        .and_hms_milli_opt(10, 16, 11, 11)
                        .unwrap()
                        .and_utc(),
                ),
                origination_expire_date_time: Some(
                    NaiveDate::from_ymd_opt(2024, 12, 24)
                        .unwrap()
                        .and_hms_milli_opt(10, 16, 11, 11)
                        .unwrap()
                        .and_utc(),
                ),
                usage_expire_date_time: Some(
                    NaiveDate::from_ymd_opt(2024, 12, 24)
                        .unwrap()
                        .and_hms_milli_opt(10, 16, 11, 11)
                        .unwrap()
                        .and_utc(),
                ),
                usage_count_limit: Some(3),
                no_auth_required: true,
                user_auth_type: Some(HardwareAuthenticatorType(3)),
                auth_timeout: Some(Duration::from_secs(86400)),
                allow_while_on_body: true,
                trusted_user_presence_required: true,
                trusted_confirmation_required: true,
                unlocked_device_required: true,
                all_applications: true,
                creation_date_time: Some(
                    NaiveDate::from_ymd_opt(2024, 12, 24)
                        .unwrap()
                        .and_hms_milli_opt(10, 16, 11, 11)
                        .unwrap()
                        .and_utc(),
                ),
                origin: Some(KeyOrigin::Generated),
                root_of_trust: Some(RootOfTrust {
                    verified_boot_key: OctetString::copy_from_slice(
                        b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    device_locked: false,
                    verified_boot_state: VerifiedBootState::Unverified,
                    verified_boot_hash: OctetString::copy_from_slice(
                        b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                }),
                os_version: Some(OsVersion { major: 13, minor: 0, sub_minor: 0}),
                os_patch_level: Some(PatchLevel::new(2024, 3, None)),
                attestation_application_id: Some(AttestationApplicationId {
                    package_infos: SetOf::from_vec(vec![AttestationPackageInfo {
                        package_name: OctetString::copy_from_slice(
                            b"nl.rijksoverheid.edi.wallet.platform_support.test",
                        ),
                        version: 0.into(),
                    }]),
                    signature_digests: SetOf::from_vec(vec![
                        OctetString::copy_from_slice(b"\xd3\xa5O\x11T\xc2ZZ\xb3\xf1%(\xdc\xc3r.\x0b\x8e\n\xd8\x11\xd42T\x84\xb7\xb2+\x0e\x8a\x1f\xe3"),
                    ]),
                }),
                attestation_id_brand: Some(OctetString::copy_from_slice(b"attestation_id_brand")),
                attestation_id_device: Some(OctetString::copy_from_slice(b"attestation_id_device")),
                attestation_id_product: Some(OctetString::copy_from_slice(b"attestation_id_product")),
                attestation_id_serial: Some(OctetString::copy_from_slice(b"attestation_id_serial")),
                attestation_id_imei: Some(OctetString::copy_from_slice(b"attestation_id_imei")),
                attestation_id_meid: Some(OctetString::copy_from_slice(b"attestation_id_meid")),
                attestation_id_manufacturer: Some(OctetString::copy_from_slice(b"attestation_id_manufacturer")),
                attestation_id_model: Some(OctetString::copy_from_slice(b"attestation_id_model")),
                vendor_patch_level: Some(PatchLevel::new(0, 0, None)),
                boot_patch_level: Some(PatchLevel::new(2024, 3, Some(1))),
                device_unique_attestation: true,
                attestation_id_second_imei: Some(OctetString::copy_from_slice(b"attestation_id_second_imei")),
            },
            hardware_enforced: AuthorizationList::default(),
        };

        assert_eq!(actual, expected);
    }
}
