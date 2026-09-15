use std::collections::HashSet;
use std::num::NonZeroU32;
use std::num::NonZeroU64;
use std::time::Duration as StdDuration;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::registration_certificate::ParsedRegistrationCertificate;
use attestation_data::x509::RelyingParty;
use attestation_types::claim_path::ClaimPath;
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use chrono::Duration;
use chrono::Utc;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use clio::CachedInput;
use cose::wrprc_cwt::SignedWrprcCwt;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate;
use crypto::x509::BorrowingCertificateExtension;
use crypto::x509::CertificateConfiguration;
use crypto::x509::CertificateUsage;
use crypto::x509::DistinguishedName;
use crypto::x509::NO_SAN;
use crypto::x509::SubjectAltNameUri;
use indexmap::IndexMap;
use itertools::Itertools;
use jwt::SignedJwt;
use jwt::jades_b_b::JadesbbHeader;
use mdoc::DataElements;
use mdoc::DeviceRequest;
use mdoc::ItemsRequest;
use mdoc::NameSpaces;
use mdoc::SessionTranscript;
use mdoc::holder::disclosure::create_doc_request;
use mdoc::utils::serialization::cbor_serialize;
use rcgen::RevokedCertParams;
use rcgen::SerialNumber;
use serde::Serialize;
use serde_json::Value;
use time::OffsetDateTime;
use token_status_list::status_list::StatusList as TokenStatusList;
use token_status_list::status_list::StatusType;
use token_status_list::status_list_token::StatusListToken;
use url::Url;
use utils::built_info::version_string;
use utils::generator::TimeGenerator;
use utils::vec_at_least::VecNonEmpty;
use wallet_ca::next_crl_number;
use wallet_ca::read_certificate;
use wallet_ca::read_key_pair;
use wallet_ca::read_public_key;
use wallet_ca::read_self_signed_ca;
use wallet_ca::write_certificate;
use wallet_ca::write_crl;
use wallet_ca::write_key_pair;

/// Generate private keys and certificates
///
/// NOTE: Do NOT use in production environments. Certificate lifetimes are large by default, and while `crl` can
/// generate Certificate Revocation Lists, nothing here operates revocation as a live service (periodically
/// regenerating and republishing CRLs as certificates get revoked).
#[derive(Parser)]
#[command(author, version=version_string(), about, long_about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, ValueEnum)]
enum CertType {
    /// Mdoc/mdl issuer certificate; requires --issuer-auth-file
    Issuer,
    /// Token Status List signing certificate
    Tsl,
    /// Wallet Issuer Authentication certificate
    Wia,
    /// Wallet Relying Party Access Certificate (WRPAC)
    Wrpac,
    /// Wallet Relying Party Registration Certificate (WRPRC) signing certificate
    Wrprc,
}

#[derive(Clone, Copy, ValueEnum)]
enum RegistrationCertificateFormat {
    /// JAdES B-B JWT serialization
    Jwt,
    /// COSE-signed WRPRC CWT serialization
    Cwt,
}

#[derive(Clone, Copy, ValueEnum)]
enum StatusListEntry {
    /// The referenced token is valid
    Valid,
    /// The referenced token is revoked
    Revoked,
    /// The referenced token is temporarily invalid
    Suspended,
}

impl From<StatusListEntry> for StatusType {
    fn from(value: StatusListEntry) -> Self {
        match value {
            StatusListEntry::Valid => StatusType::Valid,
            StatusListEntry::Revoked => StatusType::Invalid,
            StatusListEntry::Suspended => StatusType::Suspended,
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// Generate a private key and certificate to use as Certificate Authority (CA)
    Ca {
        /// Subject Common Name to use in the new certificate
        #[arg(short = 'n', long)]
        common_name: String,
        /// Subject Country name to use in the new certificate when not NL
        #[arg(long)]
        country_name: Option<String>,
        /// Prefix to use for the generated files: <FILE_PREFIX>.key.pem and <FILE_PREFIX>.crt.pem
        #[arg(short, long)]
        file_prefix: String,
        /// Duration for which the certificate will be valid
        #[arg(short, long, default_value = "3650")]
        days: u32,
        /// Overwrite existing files
        #[arg(long, default_value = "false")]
        force: bool,
    },
    /// Generate a private key and certificate signed by given Certificate Authority (CA)
    Cert {
        /// Path to the CA key file in PEM format
        #[arg(short = 'k', long, value_parser)]
        ca_key_file: CachedInput,
        /// Path to the CA certificate file in PEM format
        #[arg(short = 'c', long, value_parser)]
        ca_crt_file: CachedInput,
        /// CRL Distribution Point URL(s) to embed in this certificate. Each URL must serve a CRL, signed by the
        /// given CA, covering this certificate's serial number; see the `crl` subcommand. If omitted, the
        /// certificate carries no CDP extension, so consumers that enforce revocation checking will reject it.
        #[arg(short = 'C', long = "crl-distribution-point", num_args(0..))]
        crl_distribution_points: Vec<Url>,
        /// Subject Common Name to use in the new certificate
        #[arg(short = 'n', long)]
        common_name: String,
        /// Subject Country name to use in the new certificate when not NL
        #[arg(long)]
        country_name: Option<String>,
        /// Subject Organization name to use in the new certificate when different from name
        #[arg(long)]
        organization_name: Option<String>,
        /// Subject Organization identifier to use in the new certificate
        #[arg(long)]
        organization_id: Option<String>,
        /// Subject Serial Number identifier to use in the new certificate
        #[arg(long)]
        serial_number: Option<String>,
        /// Subject Surname identifier to use in the new certificate
        #[arg(long)]
        surname: Option<String>,
        /// Subject Given Name identifier to use in the new certificate
        #[arg(long)]
        given_name: Option<String>,
        /// Subject Alternative Name URIs
        #[arg(long = "san-uri", num_args(0..))]
        san_uris: Vec<String>,
        /// Certificate type in EDI
        #[arg(short = 't', long = "type", value_parser)]
        cert_type: CertType,
        /// Path to Issuer Authentication file in JSON format
        #[arg(short, long, value_parser)]
        issuer_auth_file: Option<CachedInput>,
        /// Prefix to use for the generated files: <FILE_PREFIX>.key.pem and <FILE_PREFIX>.crt.pem
        #[arg(short, long)]
        file_prefix: String,
        /// Duration for which the certificate will be valid
        #[arg(short, long, default_value = "365")]
        days: u32,
        /// Overwrite existing files
        #[arg(long, default_value = "false")]
        force: bool,
    },
    /// Generate a certificate based on a public key signed by given Certificate Authority (CA)
    CertPub {
        /// Path to the public key for which the certificate should be generated
        #[arg(short = 'p', long, value_parser)]
        public_key_file: CachedInput,
        /// Path to the CA key file in PEM format
        #[arg(short = 'k', long, value_parser)]
        ca_key_file: CachedInput,
        /// Path to the CA certificate file in PEM format
        #[arg(short = 'c', long, value_parser)]
        ca_crt_file: CachedInput,
        /// CRL Distribution Point URL(s) to embed in this certificate. Each URL must serve a CRL, signed by the
        /// given CA, covering this certificate's serial number; see the `crl` subcommand. If omitted, the
        /// certificate carries no CDP extension, so consumers that enforce revocation checking will reject it.
        #[arg(short = 'C', long = "crl-distribution-point", num_args(0..))]
        crl_distribution_points: Vec<Url>,
        /// Subject Common Name to use in the new certificate
        #[arg(short = 'n', long)]
        common_name: String,
        /// Subject Country name to use in the new certificate when not NL
        #[arg(long)]
        country_name: Option<String>,
        /// Subject Organization name to use in the new certificate
        #[arg(long)]
        organization_name: Option<String>,
        /// Subject Organization identifier to use in the new certificate
        #[arg(long)]
        organization_id: Option<String>,
        /// Subject Serial Number identifier to use in the new certificate
        #[arg(long)]
        serial_number: Option<String>,
        /// Subject Surname identifier to use in the new certificate
        #[arg(long)]
        surname: Option<String>,
        /// Subject Given Name identifier to use in the new certificate
        #[arg(long)]
        given_name: Option<String>,
        /// Subject Alternative Name URIs
        #[arg(long = "san-uri", num_args(0..))]
        san_uris: Vec<String>,
        /// Certificate type in EDI
        #[arg(short = 't', long = "type", value_parser)]
        cert_type: CertType,
        /// Path to Issuer Authentication file in JSON format
        #[arg(short, long, value_parser)]
        issuer_auth_file: Option<CachedInput>,
        /// Prefix to use for the generated files: <FILE_PREFIX>.crt.pem
        #[arg(short, long)]
        file_prefix: String,
        /// Duration for which the certificate will be valid
        #[arg(short, long, default_value = "365")]
        days: u32,
        /// Overwrite existing files
        #[arg(long, default_value = "false")]
        force: bool,
    },
    /// Sign a WRPRC payload and print its unpadded base64url serialization
    RegistrationCertificate {
        /// Path to the WRPRC signing key file in PEM format
        #[arg(long, value_parser)]
        wrprc_key_file: CachedInput,
        /// Path to the WRPRC signing certificate file in PEM format
        #[arg(long, value_parser)]
        wrprc_crt_file: CachedInput,
        /// Path to the WRPAC whose subject must match the WRPRC payload in PEM format
        #[arg(long, value_parser)]
        wrpac_crt_file: CachedInput,
        /// Path to the unsigned WRPRC payload in JSON format
        #[arg(long, value_parser)]
        payload_file: CachedInput,
        /// WRPRC envelope format
        #[arg(long, value_enum)]
        format: RegistrationCertificateFormat,
    },
    /// Sign a Status List Token and print its compact JWT serialization
    StatusList {
        /// Path to the Token Status List signing key file in PEM format
        #[arg(long, value_parser)]
        tsl_key_file: CachedInput,
        /// Path to the Token Status List signing certificate file in PEM format
        #[arg(long, value_parser)]
        tsl_crt_file: CachedInput,
        /// Public URI from which this exact Status List Token will be served
        #[arg(long)]
        uri: Url,
        /// Status of each referenced token, in index order
        #[arg(long = "status", value_enum, required = true, num_args = 1..)]
        statuses: Vec<StatusListEntry>,
        /// Number of days after issuance at which the token expires; omit for no explicit expiration
        #[arg(long)]
        valid_for_days: Option<NonZeroU32>,
        /// Maximum number of seconds for which a consumer should cache the token; omit to leave out the TTL claim
        #[arg(long)]
        ttl_seconds: Option<NonZeroU64>,
    },
    /// Generate a signed mdoc DeviceRequest for close-proximity disclosure
    ReaderDeviceRequest {
        /// Path to the CA key file in PEM format
        #[arg(short = 'k', long, value_parser)]
        ca_key_file: CachedInput,
        /// Path to the CA certificate file in PEM format
        #[arg(short = 'c', long, value_parser)]
        ca_crt_file: CachedInput,
        /// Subject Common Name to use in the new certificate
        #[arg(short = 'n', long)]
        common_name: String,
        /// Subject Country name to use in the new certificate when not NL
        #[arg(long)]
        country_name: Option<String>,
        /// Subject Organization name to use in the new certificate
        #[arg(long)]
        organization_name: Option<String>,
        /// Subject Organization identifier to use in the new certificate
        #[arg(long)]
        organization_id: Option<String>,
        /// Subject Serial Number identifier to use in the new certificate
        #[arg(long)]
        serial_number: Option<String>,
        /// Subject Surname identifier to use in the new certificate
        #[arg(long)]
        surname: Option<String>,
        /// Subject Given Name identifier to use in the new certificate
        #[arg(long)]
        given_name: Option<String>,
        /// Hex-encoded CBOR SessionTranscript
        #[arg(long)]
        session_transcript_hex: String,
    },
    /// Generate a CRL, signed by the CA
    ///
    /// `crlNumber` starts at the generation time and advances from the existing PEM file on regeneration. To actually
    /// revoke a certificate, regenerate the CRL for its issuing CA with the same file prefix and that certificate's
    /// serial number added to --serial-number, then re-publish the result at the certificate's CDP URL(s).
    Crl {
        /// Path to the CA key file in PEM format
        #[arg(short = 'k', long, value_parser)]
        ca_key_file: CachedInput,
        /// Path to the CA certificate file in PEM format
        #[arg(short = 'c', long, value_parser)]
        ca_crt_file: CachedInput,
        /// Prefix for the generated file: <FILE_PREFIX>.crl.pem. Convert it to DER before publishing it at the URL
        /// embedded in certificates as a CRL Distribution Point.
        #[arg(short, long)]
        file_prefix: String,
        /// Duration for which the CRL will be valid (used to calculate `nextUpdate`); choose based on how often you
        /// intend to regenerate and republish it
        #[arg(short, long)]
        days: u32,
        /// Revoked Serial Numbers, hex-encoded (colons optional, as in `openssl x509 -noout -serial`/-text output)
        #[arg(short, long = "serial-number", num_args(0..))]
        serial_numbers: Vec<String>,
        /// Overwrite existing files
        #[arg(long, default_value = "false")]
        force: bool,
    },
}

impl Command {
    fn default_country_name() -> String {
        "NL".to_string()
    }

    #[expect(clippy::too_many_arguments, reason = "constructor like method")]
    fn get_distinguished_name(
        common_name: String,
        country_name: Option<String>,
        organization_name: Option<String>,
        organization_identifier: Option<String>,
        serial_number: Option<String>,
        surname: Option<String>,
        given_name: Option<String>,
    ) -> Result<DistinguishedName> {
        let country_name = country_name.unwrap_or_else(Self::default_country_name);
        match (
            &organization_name,
            &organization_identifier,
            &serial_number,
            &surname,
            &given_name,
        ) {
            (Some(_), Some(_), _, _, _) | (_, _, Some(_), Some(_), Some(_)) => {}
            // Only disallow names that are neither legal nor natural persons, allow extra attributes
            _ => anyhow::bail!("Illegal subject name, specify either for a legal or natural person"),
        }
        Ok(DistinguishedName {
            common_name,
            country_name,
            organization_name,
            organization_identifier,
            serial_number,
            surname,
            given_name,
        })
    }

    fn get_san_uris(uris: Vec<String>) -> Result<Vec<SubjectAltNameUri>> {
        uris.into_iter()
            .map(|uri| uri.parse::<SubjectAltNameUri>().map_err(anyhow::Error::from))
            .try_collect()
    }

    fn get_ca_configuration(days: u32) -> CertificateConfiguration {
        let not_before = Utc::now();
        let not_after = not_before
            .checked_add_signed(Duration::days(i64::from(days)))
            .expect("`valid_for` does not result in a valid time stamp, try decreasing the value");
        if not_after <= not_before {
            panic!("`valid_for` must be a positive duration");
        }
        CertificateConfiguration {
            not_before: Some(not_before),
            not_after: Some(not_after),
            ..Default::default()
        }
    }

    fn get_certificate_configuration(
        cert_type: CertType,
        issuer_auth_file: Option<CachedInput>,
        days: u32,
        crl_distribution_points: Vec<Url>,
    ) -> Result<CertificateConfiguration> {
        let usage = match cert_type {
            CertType::Issuer => Some(CertificateUsage::Mdl),
            CertType::Tsl => Some(CertificateUsage::StatusListSigning),
            CertType::Wia => Some(CertificateUsage::Wia),
            CertType::Wrpac | CertType::Wrprc => None,
        };

        let extension = issuer_auth_file
            .map(|auth_file| serde_json::from_reader::<_, IssuerRegistration>(auth_file)?.to_custom_ext())
            .transpose()?;

        Ok(CertificateConfiguration {
            usage,
            extension,
            crl_distribution_points,
            ..Self::get_ca_configuration(days)
        })
    }

    fn execute(self) -> Result<()> {
        use Command::*;
        match self {
            Ca {
                common_name,
                country_name,
                file_prefix,
                days,
                force,
            } => {
                let distinguished_name =
                    DistinguishedName::new(common_name, country_name.unwrap_or_else(Self::default_country_name));
                let configuration = Self::get_ca_configuration(days);
                let ca = generate::Ca::generate(distinguished_name, configuration)?;
                let signing_key = ca.to_signing_key()?;
                write_key_pair(ca.certificate(), &signing_key, &file_prefix, force)?;
                Ok(())
            }
            Cert {
                ca_key_file,
                ca_crt_file,
                crl_distribution_points,
                common_name,
                country_name,
                organization_name,
                serial_number,
                surname,
                given_name,
                organization_id,
                san_uris,
                cert_type,
                issuer_auth_file,
                file_prefix,
                days,
                force,
            } => {
                let ca = read_self_signed_ca(&ca_crt_file, &ca_key_file)?;

                let distinguished_name = Self::get_distinguished_name(
                    common_name,
                    country_name,
                    organization_name,
                    organization_id,
                    serial_number,
                    surname,
                    given_name,
                )?;
                let config =
                    Self::get_certificate_configuration(cert_type, issuer_auth_file, days, crl_distribution_points)?;
                let san_uris = Self::get_san_uris(san_uris)?;
                let key_pair = ca.generate_key_pair(distinguished_name, config, san_uris)?;
                write_key_pair(key_pair.certificate(), key_pair.private_key(), &file_prefix, force)?;
                Ok(())
            }
            CertPub {
                public_key_file,
                ca_key_file,
                ca_crt_file,
                crl_distribution_points,
                common_name,
                country_name,
                organization_name,
                organization_id,
                serial_number,
                surname,
                given_name,
                san_uris,
                cert_type,
                issuer_auth_file,
                file_prefix,
                days,
                force,
            } => {
                let ca = read_self_signed_ca(&ca_crt_file, &ca_key_file)?;
                let public_key = read_public_key(&public_key_file)?;

                let distinguished_name = Self::get_distinguished_name(
                    common_name,
                    country_name,
                    organization_name,
                    organization_id,
                    serial_number,
                    surname,
                    given_name,
                )?;
                let config =
                    Self::get_certificate_configuration(cert_type, issuer_auth_file, days, crl_distribution_points)?;
                let san_uris = Self::get_san_uris(san_uris)?;
                let certificate =
                    ca.generate_certificate(public_key.contents(), distinguished_name, config, san_uris)?;
                write_certificate(&certificate, &file_prefix, force)?;
                Ok(())
            }
            RegistrationCertificate {
                wrprc_key_file,
                wrprc_crt_file,
                wrpac_crt_file,
                payload_file,
                format,
            } => {
                let signing_key_pair = read_key_pair(&wrprc_crt_file, &wrprc_key_file)?;
                if signing_key_pair.certificate().x509_certificate().is_ca() {
                    anyhow::bail!("WRPRC signing certificate must be an end-entity certificate");
                }

                let access_certificate = read_certificate(&wrpac_crt_file)?;
                let access_subject = RelyingParty::try_from(access_certificate.to_distinguished_name()?)?;
                let payload: Value = serde_json::from_reader(payload_file)?;
                serde_json::from_value::<ParsedRegistrationCertificate>(payload.clone())?
                    .validate_binding_and_time(&access_subject, Utc::now())?;

                let runtime = tokio::runtime::Builder::new_current_thread().build()?;
                let serialized = runtime.block_on(sign_registration_certificate(
                    RegistrationCertificatePayload(payload),
                    &signing_key_pair,
                    format,
                ))?;

                println!("{}", BASE64_URL_SAFE_NO_PAD.encode(serialized));
                Ok(())
            }
            StatusList {
                tsl_key_file,
                tsl_crt_file,
                uri,
                statuses,
                valid_for_days,
                ttl_seconds,
            } => {
                let signing_key_pair = read_key_pair(&tsl_crt_file, &tsl_key_file)?;
                if signing_key_pair.certificate().x509_certificate().is_ca() {
                    anyhow::bail!("status list signing certificate must be an end-entity certificate");
                }
                let usage = CertificateUsage::from_certificate(signing_key_pair.certificate().x509_certificate())
                    .context("status list signing certificate must have Status List Signing usage")?;
                if usage != CertificateUsage::StatusListSigning {
                    anyhow::bail!("status list signing certificate must have Status List Signing usage");
                }

                let mut status_list = TokenStatusList::new(statuses.len());
                for (index, status) in statuses.into_iter().enumerate() {
                    let status = StatusType::from(status);
                    if status != StatusType::Valid {
                        status_list.insert(index, status);
                    }
                }

                let expiration = valid_for_days
                    .map(|days| {
                        Utc::now()
                            .checked_add_signed(Duration::days(i64::from(days.get())))
                            .context("status list token validity overflows the supported date range")
                    })
                    .transpose()?;
                let ttl = ttl_seconds.map(|seconds| StdDuration::from_secs(seconds.get()));

                let runtime = tokio::runtime::Builder::new_current_thread().build()?;
                let token = runtime.block_on(
                    StatusListToken::builder(uri, status_list.pack())
                        .exp(expiration)
                        .ttl(ttl)
                        .sign(&signing_key_pair),
                )?;

                println!("{}", token.as_ref().serialization());
                Ok(())
            }
            ReaderDeviceRequest {
                ca_key_file,
                ca_crt_file,
                common_name,
                country_name,
                organization_name,
                organization_id,
                serial_number,
                surname,
                given_name,
                session_transcript_hex,
            } => {
                let ca = read_self_signed_ca(&ca_crt_file, &ca_key_file)?;
                let distinguished_name = Self::get_distinguished_name(
                    common_name,
                    country_name,
                    organization_name,
                    organization_id,
                    serial_number,
                    surname,
                    given_name,
                )?;

                let session_transcript_bytes =
                    hex::decode(&session_transcript_hex).with_context(|| "invalid session transcript hex")?;
                let session_transcript = SessionTranscript::try_from_bytes(&session_transcript_bytes)?;
                let runtime = tokio::runtime::Builder::new_current_thread().build()?;
                let device_request = runtime.block_on(create_reader_device_request(
                    &ca,
                    distinguished_name,
                    &session_transcript,
                ))?;

                println!("{}", hex::encode(cbor_serialize(&device_request)?));
                Ok(())
            }
            Crl {
                ca_key_file,
                ca_crt_file,
                file_prefix,
                days,
                serial_numbers,
                force,
            } => {
                let ca = read_self_signed_ca(&ca_crt_file, &ca_key_file)?;

                let this_update = OffsetDateTime::now_utc();
                let next_update = this_update + time::Duration::days(i64::from(days));
                let crl_number = next_crl_number(&file_prefix, this_update)?;
                let revoked_certs = serial_numbers
                    .into_iter()
                    .map(|sn| {
                        let serial_number = hex::decode(sn.replace(':', ""))
                            .with_context(|| format!("invalid hex-encoded serial number '{sn}'"))?;
                        Ok(RevokedCertParams {
                            serial_number: SerialNumber::from(serial_number),
                            revocation_time: this_update,
                            reason_code: Some(rcgen::RevocationReason::Unspecified),
                            invalidity_date: None,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;

                let crl = ca.generate_crl_with_validity(revoked_certs, this_update, next_update, crl_number)?;

                write_crl(&file_prefix, &crl, force)?;
                Ok(())
            }
        }
    }
}

#[derive(Serialize)]
#[serde(transparent)]
struct RegistrationCertificatePayload(Value);

impl jwt::JwtTyp for RegistrationCertificatePayload {
    const TYP: &'static str = jwt::jades_b_b::JADES_B_B_JWT_TYP;
}

async fn sign_registration_certificate(
    payload: RegistrationCertificatePayload,
    signing_key_pair: &KeyPair,
    format: RegistrationCertificateFormat,
) -> Result<Vec<u8>> {
    match format {
        RegistrationCertificateFormat::Jwt => {
            Ok(
                SignedJwt::<_, JadesbbHeader>::sign_with_iat(&payload, signing_key_pair, &TimeGenerator)
                    .await?
                    .to_string()
                    .into_bytes(),
            )
        }
        RegistrationCertificateFormat::Cwt => {
            Ok(
                SignedWrprcCwt::sign_with_certificate(&payload, signing_key_pair, &TimeGenerator)
                    .await?
                    .to_vec()?,
            )
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.command.execute()?;
    Ok(())
}

fn items_requests_from_registration_cert(
    intent_to_retain: bool,
    authorized_attributes: &Vec<(String, Vec<VecNonEmpty<ClaimPath>>)>,
) -> Result<Vec<ItemsRequest>> {
    let mut items_requests = Vec::with_capacity(authorized_attributes.len());

    for (doc_type, authorized_paths) in authorized_attributes {
        let mut name_spaces = IndexMap::<String, IndexMap<String, bool>>::new();

        for authorized_path in authorized_paths {
            let key_segments = claim_path_segments(doc_type, authorized_path)?;
            let (namespace, attribute) = match key_segments.as_slice() {
                [attribute] => (doc_type.clone(), attribute.clone()),
                [namespace, attribute] if is_mdoc_namespace_for_doc_type(doc_type, namespace) => {
                    (namespace.clone(), attribute.clone())
                }
                [_, _] => continue,
                _ => {
                    return Err(anyhow!(
                        "unsupported authorized attribute path for credential type '{doc_type}': expected 1 or 2 key \
                         segments, got {}",
                        key_segments.len()
                    ));
                }
            };

            name_spaces
                .entry(namespace)
                .or_default()
                .insert(attribute, intent_to_retain);
        }

        if name_spaces.is_empty() {
            continue;
        }

        let name_spaces = name_spaces
            .into_iter()
            .map(|(namespace, data_elements)| {
                let data_elements = DataElements::try_from(data_elements)
                    .map_err(|_| anyhow!("no data elements could be derived for namespace '{namespace}'"))?;
                Ok((namespace, data_elements))
            })
            .collect::<Result<IndexMap<_, _>>>()?;
        let name_spaces = NameSpaces::try_from(name_spaces)
            .map_err(|_| anyhow!("no namespaces could be derived for credential type '{doc_type}'"))?;

        let items_request = ItemsRequest {
            doc_type: doc_type.clone(),
            name_spaces,
            request_info: None,
        };

        // Registration certificates can contain disclosure definitions for other credential formats
        // alongside mdoc requests. Only keep doc types that round-trip through the same
        // requested-attribute validation the holder uses for close-proximity disclosure.
        if authorized_paths_round_trip_as_mdoc(doc_type, authorized_paths, &items_request)? {
            items_requests.push(items_request);
        }
    }

    if items_requests.is_empty() {
        return Err(anyhow!(
            "no authorized attributes found that can be translated into an mdoc request"
        ));
    }

    Ok(items_requests)
}

fn claim_path_segments(doc_type: &str, authorized_path: &VecNonEmpty<ClaimPath>) -> Result<Vec<String>> {
    authorized_path
        .as_ref()
        .iter()
        .map(|segment| match segment {
            ClaimPath::SelectByKey(value) => Ok(value.clone()),
            ClaimPath::SelectAll | ClaimPath::SelectByIndex(_) => Err(anyhow!(
                "unsupported non-key claim path segment for credential type '{doc_type}'"
            )),
        })
        .collect()
}

fn authorized_paths_round_trip_as_mdoc(
    doc_type: &str,
    authorized_paths: &[VecNonEmpty<ClaimPath>],
    items_request: &ItemsRequest,
) -> Result<bool> {
    let mut normalized_claim_paths = HashSet::with_capacity(authorized_paths.len());

    for authorized_path in authorized_paths {
        let key_segments = claim_path_segments(doc_type, authorized_path)?;
        let claim_path: VecNonEmpty<ClaimPath> = match key_segments.as_slice() {
            [attribute] => vec![
                ClaimPath::SelectByKey(doc_type.to_string()),
                ClaimPath::SelectByKey(attribute.clone()),
            ]
            .try_into()?,
            [namespace, attribute] if is_mdoc_namespace_for_doc_type(doc_type, namespace) => vec![
                ClaimPath::SelectByKey(namespace.clone()),
                ClaimPath::SelectByKey(attribute.clone()),
            ]
            .try_into()?,
            [_, _] => return Ok(false),
            _ => {
                return Err(anyhow!(
                    "unsupported authorized attribute path for credential type '{doc_type}': expected 1 or 2 key \
                     segments, got {}",
                    key_segments.len()
                ));
            }
        };

        normalized_claim_paths.insert(claim_path);
    }

    Ok(items_request.claims().collect::<HashSet<_>>() == normalized_claim_paths)
}

fn is_mdoc_namespace_for_doc_type(doc_type: &str, namespace: &str) -> bool {
    namespace == doc_type
        || namespace
            .strip_prefix(doc_type)
            .is_some_and(|suffix| suffix.starts_with('.'))
}

async fn create_reader_device_request(
    ca: &generate::Ca,
    distinguished_name: DistinguishedName,
    session_transcript: &SessionTranscript,
) -> Result<DeviceRequest> {
    // TODO PVW-6052 Derive item requests from `credentials` field of WRPRC
    let items_requests = items_requests_from_registration_cert(true, &vec![])?;
    let key_pair = ca.generate_key_pair(distinguished_name, CertificateConfiguration::default(), NO_SAN)?;

    let mut doc_requests = Vec::with_capacity(items_requests.len());
    for items_request in items_requests {
        doc_requests.push(create_doc_request(items_request, session_transcript, &key_pair).await);
    }

    let doc_requests = VecNonEmpty::try_from(doc_requests).map_err(|_| anyhow!("empty doc requests"))?;

    Ok(DeviceRequest::from_doc_requests(doc_requests))
}
