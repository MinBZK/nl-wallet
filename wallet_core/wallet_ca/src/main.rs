use std::collections::HashSet;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::auth::reader_auth::ReaderRegistration;
use attestation_types::claim_path::ClaimPath;
use chrono::Duration;
use chrono::Utc;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use clio::CachedInput;
use crypto::server_keys::generate;
use crypto::x509::BorrowingCertificateExtension;
use crypto::x509::CertificateConfiguration;
use crypto::x509::CertificateUsage;
use crypto::x509::DistinguishedName;
use crypto::x509::SubjectAltNameUri;
use indexmap::IndexMap;
use itertools::Itertools;
use mdoc::DataElements;
use mdoc::DeviceRequest;
use mdoc::ItemsRequest;
use mdoc::NameSpaces;
use mdoc::SessionTranscript;
use mdoc::holder::disclosure::create_doc_request;
use mdoc::utils::serialization::cbor_serialize;
use utils::built_info::version_string;
use utils::vec_at_least::VecNonEmpty;
use wallet_ca::read_public_key;
use wallet_ca::read_self_signed_ca;
use wallet_ca::write_certificate;
use wallet_ca::write_key_pair;

/// Generate private keys and certificates
///
/// NOTE: Do NOT use in production environments, as the certificates lifetime is incredibly large, and no revocation is
/// implemented.
#[derive(Parser)]
#[command(author, version=version_string(), about, long_about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, ValueEnum)]
enum CertType {
    Issuer,
    Reader,
    Tsl,
    Wia,
    Wrpac,
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
        #[arg(short, long, default_value = "365")]
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
        /// Path to Reader Authentication file in JSON format
        #[arg(short, long, value_parser)]
        reader_auth_file: Option<CachedInput>,
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
        /// Path to Reader Authentication file in JSON format
        #[arg(short, long, value_parser)]
        reader_auth_file: Option<CachedInput>,
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
        /// Subject Alternative Name URIs
        #[arg(long = "san-uri", num_args(0..))]
        san_uris: Vec<String>,
        /// Path to Reader Authentication file in JSON format
        #[arg(short, long, value_parser)]
        reader_auth_file: CachedInput,
        /// Hex-encoded CBOR SessionTranscript
        #[arg(long)]
        session_transcript_hex: String,
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
        reader_auth_file: Option<CachedInput>,
        days: u32,
    ) -> Result<CertificateConfiguration> {
        let usage = match cert_type {
            CertType::Issuer => Some(CertificateUsage::Mdl),
            CertType::Reader => Some(CertificateUsage::ReaderAuth),
            CertType::Tsl => Some(CertificateUsage::OAuthStatusSigning),
            CertType::Wia => Some(CertificateUsage::Wia),
            CertType::Wrpac => None,
        };

        let extension = match (issuer_auth_file, reader_auth_file) {
            (Some(_), Some(_)) => anyhow::bail!("cannot specify both reader and issuer auth file"),
            (Some(auth_file), _) => Some(serde_json::from_reader::<_, IssuerRegistration>(auth_file)?.to_custom_ext()?),
            (_, Some(auth_file)) => Some(serde_json::from_reader::<_, ReaderRegistration>(auth_file)?.to_custom_ext()?),
            (None, None) => None,
        };

        Ok(CertificateConfiguration {
            usage,
            extension,
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
                reader_auth_file,
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
                let config = Self::get_certificate_configuration(cert_type, issuer_auth_file, reader_auth_file, days)?;
                let san_uris = Self::get_san_uris(san_uris)?;
                let key_pair = ca.generate_key_pair(distinguished_name, config, san_uris)?;
                write_key_pair(key_pair.certificate(), key_pair.private_key(), &file_prefix, force)?;
                Ok(())
            }
            CertPub {
                public_key_file,
                ca_key_file,
                ca_crt_file,
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
                reader_auth_file,
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
                let config = Self::get_certificate_configuration(cert_type, issuer_auth_file, reader_auth_file, days)?;
                let san_uris = Self::get_san_uris(san_uris)?;
                let certificate =
                    ca.generate_certificate(public_key.contents(), distinguished_name, config, san_uris)?;
                write_certificate(&certificate, &file_prefix, force)?;
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
                san_uris,
                reader_auth_file,
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
                let san_uris = Self::get_san_uris(san_uris)?;
                let reader_registration: ReaderRegistration = serde_json::from_reader(reader_auth_file)?;

                let session_transcript_bytes =
                    hex::decode(&session_transcript_hex).with_context(|| "invalid session transcript hex")?;
                let session_transcript = SessionTranscript::try_from_bytes(&session_transcript_bytes)?;
                let runtime = tokio::runtime::Builder::new_current_thread().build()?;
                let device_request = runtime.block_on(create_reader_device_request(
                    &ca,
                    distinguished_name,
                    san_uris,
                    reader_registration,
                    &session_transcript,
                ))?;

                println!("{}", hex::encode(cbor_serialize(&device_request)?));
                Ok(())
            }
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.command.execute()?;
    Ok(())
}

fn items_requests_from_reader_registration(reader_registration: &ReaderRegistration) -> Result<Vec<ItemsRequest>> {
    let intent_to_retain = reader_registration.retention_policy.intent_to_retain;
    let mut items_requests = Vec::with_capacity(reader_registration.authorized_attributes.len());

    for (doc_type, authorized_paths) in &reader_registration.authorized_attributes {
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
                        "reader_auth.json contains unsupported authorized attribute path for credential type \
                         '{doc_type}': expected 1 or 2 key segments, got {}",
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

        // Reader auth files can contain disclosure definitions for other credential formats
        // alongside mdoc requests. Only keep doc types that round-trip through the same
        // requested-attribute validation the holder uses for close-proximity disclosure.
        if reader_registration
            .verify_requested_attributes([items_request.clone()])
            .is_ok()
            || authorized_paths_round_trip_as_mdoc(doc_type, authorized_paths, &items_request)?
        {
            items_requests.push(items_request);
        }
    }

    if items_requests.is_empty() {
        return Err(anyhow!(
            "reader_auth.json does not contain any authorized attributes that can be translated into an mdoc request"
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
                "reader_auth.json contains unsupported non-key claim path segment for credential type '{doc_type}'"
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
            .try_into()
            .unwrap(),
            [namespace, attribute] if is_mdoc_namespace_for_doc_type(doc_type, namespace) => vec![
                ClaimPath::SelectByKey(namespace.clone()),
                ClaimPath::SelectByKey(attribute.clone()),
            ]
            .try_into()
            .unwrap(),
            [_, _] => return Ok(false),
            _ => {
                return Err(anyhow!(
                    "reader_auth.json contains unsupported authorized attribute path for credential type \
                     '{doc_type}': expected 1 or 2 key segments, got {}",
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
    subject_alt_name_uris: Vec<SubjectAltNameUri>,
    reader_registration: ReaderRegistration,
    session_transcript: &SessionTranscript,
) -> Result<DeviceRequest> {
    let items_requests = items_requests_from_reader_registration(&reader_registration)?;
    let key_pair = ca.generate_key_pair(
        distinguished_name,
        CertificateConfiguration::with_usage_and_extension(
            CertificateUsage::ReaderAuth,
            reader_registration.to_custom_ext()?,
        ),
        subject_alt_name_uris,
    )?;

    let mut doc_requests = Vec::with_capacity(items_requests.len());
    for items_request in items_requests {
        doc_requests.push(create_doc_request(items_request, session_transcript, &key_pair).await);
    }

    let doc_requests = VecNonEmpty::try_from(doc_requests)
        .map_err(|_| anyhow!("reader_auth.json did not produce any doc requests"))?;

    Ok(DeviceRequest::from_doc_requests(doc_requests))
}

#[cfg(test)]
mod tests {
    use attestation_data::auth::reader_auth::ReaderRegistration;
    use mdoc::ItemsRequest;

    use super::items_requests_from_reader_registration;

    #[test]
    fn test_items_requests_from_reader_registration_match_original_xyz_bank_registration() {
        let reader_registration: ReaderRegistration =
            serde_json::from_str(include_str!("../../../scripts/devenv/xyz_bank_reader_auth.json")).unwrap();
        let expected_items_request: ItemsRequest = serde_json::from_str(
            r#"
            {
                "docType": "urn:eudi:pid:nl:1",
                "nameSpaces": {
                    "urn:eudi:pid:nl:1": {
                        "given_name": true,
                        "family_name": true,
                        "birthdate": true,
                        "bsn": true
                    },
                    "urn:eudi:pid:nl:1.address": {
                        "street_address": true,
                        "house_number": true,
                        "postal_code": true
                    }
                }
            }
            "#,
        )
        .unwrap();

        let items_requests = items_requests_from_reader_registration(&reader_registration).unwrap();

        assert_eq!(items_requests, vec![expected_items_request]);
    }
}
