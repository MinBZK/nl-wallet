use x509_parser::certificate::X509Certificate;
use x509_parser::der_parser::oid;
use x509_parser::der_parser::Oid;
use x509_parser::prelude::X509Error;

use crate::attestation_version::AttestationVersion;
use crate::attestation_version::AttestationVersionError;
use crate::key_attestation::KeyDescription;

pub const KEY_ATTESTATION_EXTENSION_OID: Oid = oid!(1.3.6 .1 .4 .1 .11129 .2 .1 .17);

#[derive(Debug, thiserror::Error)]
pub enum AndroidCertificateError {
    #[error("expected a single unique extension: {0}")]
    DuplicateKeyDescription(#[source] X509Error),
    #[error("failed to parse attestation version: {0}")]
    ParsingVersion(#[from] AttestationVersionError),
    #[error("failed to parse key attestation: {0}")]
    ParsingKeyAttestation(#[from] rasn::error::DecodeError),
    #[error("unsupported attestation schema version: {0:?}")]
    UnsupportedAttestationVersion(AttestationVersion),
}

pub trait GoogleAttestationExtension {
    fn key_attestation(&self) -> Result<Option<KeyDescription>, AndroidCertificateError>;
}

impl GoogleAttestationExtension for X509Certificate<'_> {
    /// Try to parse key attestation extension from the certificate.
    fn key_attestation(&self) -> Result<Option<KeyDescription>, AndroidCertificateError> {
        let key_attestation = self
            .get_extension_unique(&KEY_ATTESTATION_EXTENSION_OID)
            .map_err(AndroidCertificateError::DuplicateKeyDescription)?
            .map(|ext| {
                let attestation_version = AttestationVersion::try_parse_from(ext.value)?;
                match attestation_version.as_u16() {
                    Some(3) | Some(4) | Some(100) | Some(200) | Some(300) => Ok(rasn::der::decode(ext.value)?),
                    Some(_) | None => Err(AndroidCertificateError::UnsupportedAttestationVersion(
                        attestation_version,
                    )),
                }
            })
            .transpose()?;

        Ok(key_attestation)
    }
}

#[cfg(test)]
mod tests {
    use rasn::types::{Integer, OctetString, SetOf};
    use rstest::rstest;
    use x509_parser::prelude::FromDer;

    use crate::key_attestation::{AuthorizationList, RootOfTrust, SecurityLevel, VerifiedBootState};

    use super::*;

    const EMULATOR_CERTIFICATE_BYTES: &[u8] = include_bytes!("../test-assets/emulator-cert.der");

    const STRONGBOX_CERTIFICATE_BYTES: &[u8] = include_bytes!("../test-assets/strongbox-cert.der");

    const TEE_CERTIFICATE_BYTES: &[u8] = include_bytes!("../test-assets/tee-cert.der");

    fn cert_from_der(bytes: &'static [u8]) -> X509Certificate<'static> {
        let (_, cert) = X509Certificate::from_der(bytes).expect("valid certificate");
        cert
    }

    fn emulator_cert() -> X509Certificate<'static> {
        cert_from_der(EMULATOR_CERTIFICATE_BYTES)
    }

    fn strongbox_cert() -> X509Certificate<'static> {
        cert_from_der(STRONGBOX_CERTIFICATE_BYTES)
    }

    fn tee_cert() -> X509Certificate<'static> {
        cert_from_der(TEE_CERTIFICATE_BYTES)
    }

    #[allow(clippy::octal_escapes)]
    fn emulator_key_description() -> KeyDescription {
        KeyDescription {
            attestation_version: 200.into(),
            attestation_security_level: SecurityLevel::Software,
            key_mint_version: 200
                .into(),
            key_mint_security_level: SecurityLevel::Software,
            attestation_challenge: OctetString::copy_from_slice(&[116, 104, 105, 115, 95, 105, 115, 95, 97, 95, 99, 104, 97, 108, 108, 101, 110, 103, 101, 95, 115, 116, 114, 105, 110, 103]),
            unique_id: OctetString::new(),
            software_enforced: AuthorizationList {
                purpose: SetOf::from_vec(vec![2.into()]).into(),
                algorithm: Integer::from(3).into(),
                key_size: Integer::from(256).into(),
                digest: SetOf::from_vec(vec![4.into()]).into(),
                ec_curve: Integer::from(1).into(),
                no_auth_required: ().into(),
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
                vendor_patch_level: Integer::from(0).into(),
                boot_patch_level: Integer::from(20240301).into(),
                ..Default::default()
            },
            hardware_enforced: AuthorizationList {
                ..Default::default()
            },
        }
    }

    fn strongbox_key_description() -> KeyDescription {
        KeyDescription {
            attestation_version: 3.into(),
            attestation_security_level: SecurityLevel::StrongBox,
            key_mint_version: 4.into(),
            key_mint_security_level: SecurityLevel::StrongBox,
            attestation_challenge: OctetString::copy_from_slice(&[97, 98, 99]),
            unique_id: OctetString::new(),
            software_enforced: AuthorizationList {
                creation_date_time: Integer::from(1561115488586_i128).into(),
                attestation_application_id: OctetString::copy_from_slice(b"0\x82\x01\xb31\x82\x01\x8b0\x0c\x04\x07android\x02\x01\x1d0\x19\x04\x14com.android.keychain\x02\x01\x1d0\x19\x04\x14com.android.settings\x02\x01\x1d0\x19\x04\x14com.qti.diagservices\x02\x01\x1d0\x1a\x04\x15com.android.dynsystem\x02\x01\x1d0\x1d\x04\x18com.android.inputdevices\x02\x01\x1d0\x1f\x04\x1acom.android.localtransport\x02\x01\x1d0\x1f\x04\x1acom.android.location.fused\x02\x01\x1d0\x1f\x04\x1acom.android.server.telecom\x02\x01\x1d0 \x04\x1bcom.android.wallpaperbackup\x02\x01\x1d0!\x04\x1ccom.google.SSRestartDetector\x02\x01\x1d0\"\x04\x1dcom.google.android.hiddenmenu\x02\x01\x010#\x04\x1ecom.android.providers.settings\x02\x01\x1d1\"\x04 0\x1a\xa3\xcb\x08\x114P\x1cE\xf1B*\xbcf\xc2B$\xfd]\xed_\xdc\x8f\x17\xe6\x97\x17o\xd8f\xaa").into(),
                ..Default::default()
            },
            hardware_enforced: AuthorizationList {
                purpose: SetOf::from_vec(vec![3.into(), 2.into()]).into(),
                algorithm: Integer::from(3).into(),
                key_size: Integer::from(256).into(),
                digest: SetOf::from_vec(vec![4.into()]).into(),
                no_auth_required: ().into(),
                origin: Integer::from(0).into(),
                root_of_trust: RootOfTrust {
                    verified_boot_key: OctetString::copy_from_slice(b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"),
                    device_locked: false,
                    verified_boot_state: VerifiedBootState::Unverified,
                    verified_boot_hash: OctetString::copy_from_slice(b"r\x8d\xb1'O\x1f\x1c\xf1W\x1d\xe48\x0b\x04\x8aUJ\xc4\xa3\x80\xe7oSU\x085)\x08J\x93x\x01"),
                }.into(),
                os_version: Integer::from(0).into(),
                os_patch_level: Integer::from(201907).into(),
                vendor_patch_level: Integer::from(20190705).into(),
                boot_patch_level:  Integer::from(20190700).into(),
                ..Default::default()
            },
        }
    }

    fn tee_key_description() -> KeyDescription {
        KeyDescription {
            attestation_version: 3.into(),
            attestation_security_level: SecurityLevel::TrustedEnvironment,
            key_mint_version: 4.into(),
            key_mint_security_level: SecurityLevel::TrustedEnvironment,
            attestation_challenge: OctetString::copy_from_slice(&[97, 98, 99]),
            unique_id: OctetString::new(),
            software_enforced: AuthorizationList {
                creation_date_time: Integer::from(1531381425477_i128).into(),
                attestation_application_id: OctetString::copy_from_slice(b"0\x82\x01\xb31\x82\x01\x8b0\x0c\x04\x07android\x02\x01\x1d0\x19\x04\x14com.android.keychain\x02\x01\x1d0\x19\x04\x14com.android.settings\x02\x01\x1d0\x19\x04\x14com.qti.diagservices\x02\x01\x1d0\x1a\x04\x15com.android.dynsystem\x02\x01\x1d0\x1d\x04\x18com.android.inputdevices\x02\x01\x1d0\x1f\x04\x1acom.android.localtransport\x02\x01\x1d0\x1f\x04\x1acom.android.location.fused\x02\x01\x1d0\x1f\x04\x1acom.android.server.telecom\x02\x01\x1d0 \x04\x1bcom.android.wallpaperbackup\x02\x01\x1d0!\x04\x1ccom.google.SSRestartDetector\x02\x01\x1d0\"\x04\x1dcom.google.android.hiddenmenu\x02\x01\x010#\x04\x1ecom.android.providers.settings\x02\x01\x1d1\"\x04 0\x1a\xa3\xcb\x08\x114P\x1cE\xf1B*\xbcf\xc2B$\xfd]\xed_\xdc\x8f\x17\xe6\x97\x17o\xd8f\xaa").into(),
                ..Default::default()
            },
            hardware_enforced: AuthorizationList {
                purpose: SetOf::from_vec(vec![3.into(), 2.into()]).into(),
                algorithm: Integer::from(3).into(),
                key_size: Integer::from(256).into(),
                digest: SetOf::from_vec(vec![4.into()]).into(),
                ec_curve: Integer::from(1).into(),
                no_auth_required: ().into(),
                origin: Integer::from(0).into(),
                root_of_trust: RootOfTrust {
                    verified_boot_key: OctetString::copy_from_slice(b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"),
                    device_locked: false,
                    verified_boot_state: VerifiedBootState::Unverified,
                    verified_boot_hash: OctetString::copy_from_slice(b"r\x8d\xb1'O\x1f\x1c\xf1W\x1d\xe48\x0b\x04\x8aUJ\xc4\xa3\x80\xe7oSU\x085)\x08J\x93x\x01"),
                }.into(),
                os_version: Integer::from(0).into(),
                os_patch_level: Integer::from(201907).into(),
                vendor_patch_level: Integer::from(201907).into(),
                boot_patch_level:  Integer::from(201907).into(),
                ..Default::default()
            },
        }
    }

    #[rstest]
    #[case::strongbox(strongbox_cert(), strongbox_key_description())]
    #[case::tee(tee_cert(), tee_key_description())]
    #[case::emulator(emulator_cert(), emulator_key_description())]
    fn test_google_certificate_extension(
        #[case] cert: X509Certificate<'static>,
        #[case] expected_key_description: KeyDescription,
    ) {
        let key_attestation = cert.key_attestation().unwrap();
        let key_description = key_attestation.unwrap();
        assert_eq!(key_description, expected_key_description);
    }
}
