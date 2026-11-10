use der::oid::ObjectIdentifier;

use super::CertificateExtendedKeyUsage;

/// The DER encoded content of the certificate policies extension (RFC 5280, section 4.2.1.4) of NL Wallet
/// certificates, which contains a single policy without qualifiers:
///
/// ```text
/// SEQUENCE {                    -- CertificatePolicies
///   SEQUENCE {                  -- PolicyInformation
///     OBJECT IDENTIFIER 2.25.150580842064059311066423755779195795505
///   }
/// }
/// ```
///
/// The policy identifier is a temporary UUID OID (ITU-T X.667) for UUID `7148d0b6-12d3-4c0e-af93-f5d209a3f831`.
/// We encode it ourselves, because the `der` crate only supports OID arcs of up to 32 bits.
const NL_WALLET_CERTIFICATE_POLICIES: &[u8] = &[
    0x30, 0x18, // SEQUENCE
    0x30, 0x16, // SEQUENCE
    0x06, 0x14, // OBJECT IDENTIFIER
    0x69, 0x81, 0xe2, 0xc8, 0xe8, 0xad, 0xc2, 0xad, 0x9a, 0xb0, 0x9d, 0xaf, 0xc9, 0xfd, 0xba, 0xa0, 0xcd, 0x8f, 0xf0,
    0x31,
];

/// The QcType of PID provider certificates, id-etsi-qct-pid (ETSI TS 119 412-6, Annex A).
const ID_ETSI_QCT_PID: ObjectIdentifier = ObjectIdentifier::new_unwrap("0.4.0.194126.1.1");

/// The QcType of wallet provider certificates, id-etsi-qct-wal (ETSI TS 119 412-6, Annex A).
const ID_ETSI_QCT_WAL: ObjectIdentifier = ObjectIdentifier::new_unwrap("0.4.0.194126.1.2");

/// The profile of a generated end-entity certificate, which determines its [`CertificateExtendedKeyUsage`] and other
/// extensions that depend on its purpose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertificateProfile {
    /// PID issuer certificate.
    PidIssuer,
    /// Issuer certificate for attestations other than PID.
    Issuer,
    /// Wallet relying party access certificate for authentication.
    Wrpac,
    /// End-entity certificate for signing wallet relying party registration certificates.
    Wrprc,
    StatusListSigning,
    Wia,
    KeyAttestation,
}

impl CertificateProfile {
    /// The extended key usage of the certificate, if the profile specifies one.
    pub fn extended_key_usage(self) -> Option<CertificateExtendedKeyUsage> {
        match self {
            // Both issuer profiles map to the mdoc issuer extended key usage of ISO/IEC 18013-5, also for SD-JWT
            // signing: issuers sign both formats with the same certificate, and SD-JWT defines no extended key usage of
            // its own.
            Self::PidIssuer | Self::Issuer => Some(CertificateExtendedKeyUsage::Mdl),
            Self::StatusListSigning => Some(CertificateExtendedKeyUsage::StatusListSigning),
            Self::Wia => Some(CertificateExtendedKeyUsage::Wia),
            Self::KeyAttestation => Some(CertificateExtendedKeyUsage::KeyAttestation),
            Self::Wrpac => Some(CertificateExtendedKeyUsage::MdlReader),
            Self::Wrprc => None,
        }
    }

    pub fn key_usages(self) -> Vec<rcgen::KeyUsagePurpose> {
        match self {
            // ISO/IEC 18013-5:2021, Table B.3 requires digitalSignature only for mdoc signers.
            // This issuer profile is also used for SD-JWT signing.
            Self::PidIssuer | Self::Issuer => vec![rcgen::KeyUsagePurpose::DigitalSignature],
            // ETSI EN 319 412-2, section 4.3.2 requires a key usage; type C is digitalSignature only.
            Self::Wrpac | Self::StatusListSigning | Self::Wia | Self::KeyAttestation => {
                vec![rcgen::KeyUsagePurpose::DigitalSignature]
            }
            // ETSI TS 119 475, section 5.2.1 requires an advanced signature/seal for WRPRCs.
            // ETSI EN 319 412-2, section 4.3.2 and EN 319 412-3, section 4.3.1 recommend type A:
            // nonRepudiation, also named contentCommitment.
            Self::Wrprc => vec![rcgen::KeyUsagePurpose::ContentCommitment],
        }
    }

    /// The DER encoded content of the certificate policies extension, see [`NL_WALLET_CERTIFICATE_POLICIES`].
    ///
    /// ETSI EN 319 412-2, GEN-4.3.3-2 requires the identifier of at least one certificate policy. This applies to PID
    /// provider certificates through ETSI TS 119 412-6, PID-4.1-01, to wallet provider certificates through WAL-5.1-01
    /// and to other attestation provider certificates through EAA-6.1-02, as ETSI EN 319 412-3 builds on
    /// ETSI EN 319 412-2.
    ///
    /// ETSI TS 119 411-8, GEN-6.6.1-03 also requires a policy identifier in WRPACs and permits a provider-assigned OID.
    ///
    /// WRPRC signing certificates are modeled on ETSI EN 319 412-3, which inherits the policy requirement through
    /// LEG-4.1-1. They reuse the temporary NL Wallet policy.
    pub fn certificate_policies(self) -> Option<&'static [u8]> {
        match self {
            Self::PidIssuer | Self::Issuer | Self::Wrpac | Self::Wrprc | Self::Wia | Self::KeyAttestation => {
                Some(NL_WALLET_CERTIFICATE_POLICIES)
            }
            Self::StatusListSigning => None,
        }
    }

    /// The value of the QcType statement (ETSI EN 319 412-5, section 4.2.3), see [`ID_ETSI_QCT_PID`] and
    /// [`ID_ETSI_QCT_WAL`].
    ///
    /// ETSI TS 119 412-6, PID-4.5-01 requires the QcType statement with the value id-etsi-qct-pid in PID provider
    /// certificates, and WAL-5.2-01 requires it with the value id-etsi-qct-wal in wallet provider certificates.
    pub fn qc_type(self) -> Option<ObjectIdentifier> {
        match self {
            Self::PidIssuer => Some(ID_ETSI_QCT_PID),
            Self::Wia | Self::KeyAttestation => Some(ID_ETSI_QCT_WAL),
            Self::Issuer | Self::Wrpac | Self::Wrprc | Self::StatusListSigning => None,
        }
    }
}
