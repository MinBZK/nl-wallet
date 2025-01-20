pub mod key_attestation;
pub mod key_description;

use x509_parser::certificate::X509Certificate;
use x509_parser::der_parser::oid;
use x509_parser::der_parser::Oid;

use key_attestation::KeyAttestation;
use key_attestation::KeyDescriptionFieldError;
use key_description::KeyDescription;

#[rustfmt::skip]
pub const KEY_ATTESTATION_EXTENSION_OID: Oid = oid!(1.3.6.1.4.1.11129.2.1.17);

#[derive(Debug, thiserror::Error)]
pub enum KeyAttestationExtensionError {
    #[error("expected a single unique extension")]
    DuplicateKeyDescription,
    #[error("failed to parse key attestation: {0}")]
    ParsingKeyAttestation(#[from] rasn::error::DecodeError),
    #[error("failed to convert key description: {0}")]
    Conversion(#[from] KeyDescriptionFieldError),
}

pub trait KeyAttestationExtension {
    fn parse_key_description(&self) -> Result<Option<KeyAttestation>, KeyAttestationExtensionError>;
}

impl KeyAttestationExtension for X509Certificate<'_> {
    /// Try to parse key attestation extension from the certificate.
    fn parse_key_description(&self) -> Result<Option<KeyAttestation>, KeyAttestationExtensionError> {
        let key_description = self
            .get_extension_unique(&KEY_ATTESTATION_EXTENSION_OID)
            .map_err(|_| KeyAttestationExtensionError::DuplicateKeyDescription)?
            .map(|ext| rasn::der::decode::<KeyDescription>(ext.value))
            .transpose()?
            .map(TryInto::try_into)
            .transpose()?;

        Ok(key_description)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rasn::types::OctetString;
    use rasn::types::SetOf;
    use rstest::rstest;
    use x509_parser::prelude::FromDer;

    use super::key_attestation::Algorithm;
    use super::key_attestation::AttestationApplicationId;
    use super::key_attestation::AttestationPackageInfo;
    use super::key_attestation::AttestationVersion;
    use super::key_attestation::AuthorizationList;
    use super::key_attestation::Digest;
    use super::key_attestation::EcCurve;
    use super::key_attestation::KeyMintVersion;
    use super::key_attestation::KeyOrigin;
    use super::key_attestation::KeyPurpose;
    use super::key_attestation::OsVersion;
    use super::key_attestation::PatchLevel;
    use super::key_description::RootOfTrust;
    use super::key_description::SecurityLevel;
    use super::key_description::VerifiedBootState;

    use super::*;

    // This cert was exported from the emulator
    const EMULATOR_CERTIFICATE_BYTES: &[u8] = include_bytes!("../../test-assets/emulator-cert.der");

    // This cert was taken from the Google repo: https://github.com/google/android-key-attestation
    const STRONGBOX_CERTIFICATE_BYTES: &[u8] = include_bytes!("../../test-assets/strongbox-cert.der");

    // This cert was taken from the Google repo: https://github.com/google/android-key-attestation
    const TEE_CERTIFICATE_BYTES: &[u8] = include_bytes!("../../test-assets/tee-cert.der");

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
    fn emulator_key_attestation() -> KeyAttestation {
        KeyAttestation {
            attestation_version: AttestationVersion::V200,
            attestation_security_level: SecurityLevel::Software,
            key_mint_version: KeyMintVersion::V200,
            key_mint_security_level: SecurityLevel::Software,
            attestation_challenge: OctetString::copy_from_slice(b"this_is_a_challenge_string"),
            unique_id: OctetString::copy_from_slice(b""),
            software_enforced: AuthorizationList {
                purpose: Some(vec![KeyPurpose::Sign].into_iter().collect()),
                algorithm: Algorithm::Ec.into(),
                key_size: 256.into(),
                digest: Some(vec![Digest::Sha2_256].into_iter().collect()),
                ec_curve: EcCurve::P256.into(),
                no_auth_required: true,
                creation_date_time: Some(
                    NaiveDate::from_ymd_opt(2024, 12, 24)
                        .unwrap()
                        .and_hms_milli_opt(10, 16, 11, 11)
                        .unwrap()
                        .and_utc(),
                ),
                origin: KeyOrigin::Generated.into(),
                root_of_trust: RootOfTrust {
                    verified_boot_key: OctetString::copy_from_slice(b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"),
                    device_locked: false,
                    verified_boot_state: VerifiedBootState::Unverified,
                    verified_boot_hash: OctetString::copy_from_slice(b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"),
                }.into(),
                os_version: OsVersion::new(13, 0, 0).into(),
                os_patch_level: PatchLevel::new(2024, 3, None).into(),
                attestation_application_id: Some(
                    AttestationApplicationId {
                        package_infos: SetOf::from_vec(vec![
                            AttestationPackageInfo {
                                package_name: OctetString::copy_from_slice(b"nl.rijksoverheid.edi.wallet.platform_support.test"),
                                version: 0.into(),
                            },
                        ]),
                        signature_digests: SetOf::from_vec(vec![
                            OctetString::copy_from_slice(
                                b"\xd3\xa5O\x11T\xc2ZZ\xb3\xf1%(\xdc\xc3r.\x0b\x8e\n\xd8\x11\xd42T\x84\xb7\xb2+\x0e\x8a\x1f\xe3",
                            ),
                        ]),
                    },
                ),
                vendor_patch_level: PatchLevel::new(0, 0, None).into(),
                boot_patch_level: PatchLevel::new(2024, 3, 1.into()).into(),
                ..Default::default()
            },
            hardware_enforced: AuthorizationList {
                ..Default::default()
            },
        }
    }

    fn strongbox_key_attestation() -> KeyAttestation {
        KeyAttestation {
            attestation_version: AttestationVersion::V3,
            attestation_security_level: SecurityLevel::StrongBox,
            key_mint_version: KeyMintVersion::V4,
            key_mint_security_level: SecurityLevel::StrongBox,
            attestation_challenge: OctetString::copy_from_slice(b"abc"),
            unique_id: OctetString::copy_from_slice(b""),
            software_enforced: AuthorizationList {
                creation_date_time: Some(
                    NaiveDate::from_ymd_opt(2019, 6, 21)
                        .unwrap()
                        .and_hms_milli_opt(11, 11, 28, 586)
                        .unwrap()
                        .and_utc(),
                ),
                attestation_application_id: Some(AttestationApplicationId {
                    package_infos: SetOf::from_vec(vec![
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.dynsystem"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.qti.diagservices"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.localtransport"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.inputdevices"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.location.fused"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.server.telecom"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.wallpaperbackup"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.google.SSRestartDetector"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.keychain"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.settings"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.providers.settings"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"android"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.google.android.hiddenmenu"),
                            version: 1.into(),
                        },
                    ]),
                    signature_digests: SetOf::from_vec(vec![OctetString::copy_from_slice(
                        b"0\x1a\xa3\xcb\x08\x114P\x1cE\xf1B*\xbcf\xc2B$\xfd]\xed_\xdc\x8f\x17\xe6\x97\x17o\xd8f\xaa",
                    )]),
                }),
                ..Default::default()
            },
            hardware_enforced: AuthorizationList {
                purpose: Some(vec![KeyPurpose::Verify, KeyPurpose::Sign].into_iter().collect()),
                algorithm: Algorithm::Ec.into(),
                key_size: 256.into(),
                digest: Some(vec![Digest::Sha2_256].into_iter().collect()),
                no_auth_required: true,
                origin: KeyOrigin::Generated.into(),
                root_of_trust: RootOfTrust {
                    verified_boot_key: OctetString::copy_from_slice(
                        b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    device_locked: false,
                    verified_boot_state: VerifiedBootState::Unverified,
                    verified_boot_hash: OctetString::copy_from_slice(
                        b"r\x8d\xb1'O\x1f\x1c\xf1W\x1d\xe48\x0b\x04\x8aUJ\xc4\xa3\x80\xe7oSU\x085)\x08J\x93x\x01",
                    ),
                }
                .into(),
                os_version: OsVersion::new(0, 0, 0).into(),
                os_patch_level: PatchLevel::new(2019, 7, None).into(),
                vendor_patch_level: PatchLevel::new(2019, 7, 5.into()).into(),
                boot_patch_level: PatchLevel::new(2019, 7, 0.into()).into(),
                ..Default::default()
            },
        }
    }

    fn tee_key_attestation() -> KeyAttestation {
        KeyAttestation {
            attestation_version: AttestationVersion::V3,
            attestation_security_level: SecurityLevel::TrustedEnvironment,
            key_mint_version: KeyMintVersion::V4,
            key_mint_security_level: SecurityLevel::TrustedEnvironment,
            attestation_challenge: OctetString::copy_from_slice(b"abc"),
            unique_id: OctetString::copy_from_slice(b""),
            software_enforced: AuthorizationList {
                creation_date_time: Some(
                    NaiveDate::from_ymd_opt(2018, 7, 12)
                        .unwrap()
                        .and_hms_milli_opt(7, 43, 45, 477)
                        .unwrap()
                        .and_utc(),
                ),
                attestation_application_id: Some(AttestationApplicationId {
                    package_infos: SetOf::from_vec(vec![
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.dynsystem"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.qti.diagservices"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.localtransport"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.inputdevices"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.location.fused"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.server.telecom"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.wallpaperbackup"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.google.SSRestartDetector"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.keychain"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.settings"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.android.providers.settings"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"android"),
                            version: 29.into(),
                        },
                        AttestationPackageInfo {
                            package_name: OctetString::copy_from_slice(b"com.google.android.hiddenmenu"),
                            version: 1.into(),
                        },
                    ]),
                    signature_digests: SetOf::from_vec(vec![OctetString::copy_from_slice(
                        b"0\x1a\xa3\xcb\x08\x114P\x1cE\xf1B*\xbcf\xc2B$\xfd]\xed_\xdc\x8f\x17\xe6\x97\x17o\xd8f\xaa",
                    )]),
                }),
                ..Default::default()
            },
            hardware_enforced: AuthorizationList {
                purpose: Some(vec![KeyPurpose::Verify, KeyPurpose::Sign].into_iter().collect()),
                algorithm: Algorithm::Ec.into(),
                key_size: 256.into(),
                digest: Some(vec![Digest::Sha2_256].into_iter().collect()),
                ec_curve: EcCurve::P256.into(),
                no_auth_required: true,
                origin: KeyOrigin::Generated.into(),
                root_of_trust: RootOfTrust {
                    verified_boot_key: OctetString::copy_from_slice(
                        b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    device_locked: false,
                    verified_boot_state: VerifiedBootState::Unverified,
                    verified_boot_hash: OctetString::copy_from_slice(
                        b"r\x8d\xb1'O\x1f\x1c\xf1W\x1d\xe48\x0b\x04\x8aUJ\xc4\xa3\x80\xe7oSU\x085)\x08J\x93x\x01",
                    ),
                }
                .into(),
                os_version: OsVersion::new(0, 0, 0).into(),
                os_patch_level: PatchLevel::new(2019, 7, None).into(),
                vendor_patch_level: PatchLevel::new(2019, 7, None).into(),
                boot_patch_level: PatchLevel::new(2019, 7, None).into(),
                ..Default::default()
            },
        }
    }

    #[rstest]
    #[case::strongbox(strongbox_cert(), strongbox_key_attestation())]
    #[case::tee(tee_cert(), tee_key_attestation())]
    #[case::emulator(emulator_cert(), emulator_key_attestation())]
    fn test_google_certificate_extension(
        #[case] cert: X509Certificate<'static>,
        #[case] expected_key_attestation: KeyAttestation,
    ) {
        let key_description = cert.parse_key_description().unwrap();
        let key_attestation = key_description.unwrap();
        assert_eq!(key_attestation, expected_key_attestation);
    }
}
