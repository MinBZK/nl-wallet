use std::collections::HashSet;
use std::future::Future;

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
use clio::CachedInput;
use crypto::server_keys::generate;
use crypto::x509::CertificateConfiguration;
use crypto::x509::CertificateUsage::OAuthStatusSigning;
use indexmap::IndexMap;
use mdoc::DataElements;
use mdoc::DeviceRequest;
use mdoc::DocRequest;
use mdoc::ItemsRequest;
use mdoc::ItemsRequestBytes;
use mdoc::NameSpaces;
use mdoc::ReaderAuthenticationBytes;
use mdoc::ReaderAuthenticationKeyed;
use mdoc::SessionTranscript;
use mdoc::utils::cose::MdocCose;
use mdoc::utils::cose::new_certificate_header;
use mdoc::utils::serialization::CborSeq;
use mdoc::utils::serialization::TaggedBytes;
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

#[derive(Subcommand)]
enum Command {
    /// Generate a private key and certificate to use as Certificate Authority (CA)
    Ca {
        /// Subject Common Name to use in the new certificate
        #[arg(short = 'n', long)]
        common_name: String,
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
    /// Generate an Mdl private key and certificate
    Issuer {
        /// Path to the CA key file in PEM format
        #[arg(short = 'k', long, value_parser)]
        ca_key_file: CachedInput,
        /// Path to the CA certificate file in PEM format
        #[arg(short = 'c', long, value_parser)]
        ca_crt_file: CachedInput,
        /// Subject Common Name to use in the new certificate
        #[arg(short = 'n', long)]
        common_name: String,
        /// Path to Issuer Authentication file in JSON format
        #[arg(short, long, value_parser)]
        issuer_auth_file: CachedInput,
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
    /// Generate a private key and certificate for Relying Party Authentication
    Reader {
        /// Path to the CA key file in PEM format
        #[arg(short = 'k', long, value_parser)]
        ca_key_file: CachedInput,
        /// Path to the CA certificate file in PEM format
        #[arg(short = 'c', long, value_parser)]
        ca_crt_file: CachedInput,
        /// Subject Common Name to use in the new certificate
        #[arg(short = 'n', long)]
        common_name: String,
        /// Path to Reader Authentication file in JSON format
        #[arg(short, long, value_parser)]
        reader_auth_file: CachedInput,
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
    /// Generate a signed mdoc DeviceRequest for close-proximity disclosure
    ReaderDeviceRequest {
        /// Path to the CA key file in PEM format
        #[arg(short = 'k', long, value_parser)]
        ca_key_file: CachedInput,
        /// Path to the CA certificate file in PEM format
        #[arg(short = 'c', long, value_parser)]
        ca_crt_file: CachedInput,
        /// Optional subject Common Name to use in the ephemeral reader certificate.
        /// Defaults to the host from requestOriginBaseUrl in reader_auth.json.
        #[arg(short = 'n', long)]
        common_name: Option<String>,
        /// Path to Reader Authentication file in JSON format
        #[arg(short, long, value_parser)]
        reader_auth_file: CachedInput,
        /// Hex-encoded CBOR SessionTranscript
        #[arg(long)]
        session_transcript_hex: String,
    },
    /// Generate a Token Status List private key and certificate
    Tsl {
        /// Path to the CA key file in PEM format
        #[arg(short = 'k', long, value_parser)]
        ca_key_file: CachedInput,
        /// Path to the CA certificate file in PEM format
        #[arg(short = 'c', long, value_parser)]
        ca_crt_file: CachedInput,
        /// Subject Common Name to use in the new certificate
        #[arg(short = 'n', long)]
        common_name: String,
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
    /// Generate an Mdl certificate based on a public key
    IssuerCert {
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
        /// Path to Issuer Authentication file in JSON format
        #[arg(short, long, value_parser)]
        issuer_auth_file: CachedInput,
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
    /// Generate a certificate for Relying Party Authentication based on a public key
    ReaderCert {
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
        /// Path to Reader Authentication file in JSON format
        #[arg(short, long, value_parser)]
        reader_auth_file: CachedInput,
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
    /// Generate a Token Status List certificate based on a public key
    TslCert {
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
}

impl Command {
    fn get_certificate_configuration(days: u32) -> CertificateConfiguration {
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

    fn execute(self) -> Result<()> {
        use Command::*;
        match self {
            Ca {
                common_name,
                file_prefix,
                days,
                force,
            } => {
                let configuration = Self::get_certificate_configuration(days);
                let ca = generate::Ca::generate(&common_name, configuration)?;
                let signing_key = ca.to_signing_key()?;
                write_key_pair(ca.certificate(), &signing_key, &file_prefix, force)?;
                Ok(())
            }
            Issuer {
                ca_key_file,
                ca_crt_file,
                common_name,
                issuer_auth_file,
                file_prefix,
                days,
                force,
            } => {
                let ca = read_self_signed_ca(&ca_crt_file, &ca_key_file)?;
                let issuer_registration: IssuerRegistration = serde_json::from_reader(issuer_auth_file)?;
                let key_pair = ca.generate_key_pair(
                    &common_name,
                    issuer_registration,
                    Self::get_certificate_configuration(days),
                )?;
                write_key_pair(key_pair.certificate(), key_pair.private_key(), &file_prefix, force)?;
                Ok(())
            }
            Reader {
                ca_key_file,
                ca_crt_file,
                common_name,
                reader_auth_file,
                file_prefix,
                days,
                force,
            } => {
                let ca = read_self_signed_ca(&ca_crt_file, &ca_key_file)?;
                let reader_registration: ReaderRegistration = serde_json::from_reader(reader_auth_file)?;
                let key_pair = ca.generate_key_pair(
                    &common_name,
                    reader_registration,
                    Self::get_certificate_configuration(days),
                )?;
                write_key_pair(key_pair.certificate(), key_pair.private_key(), &file_prefix, force)?;
                Ok(())
            }
            ReaderDeviceRequest {
                ca_key_file,
                ca_crt_file,
                common_name,
                reader_auth_file,
                session_transcript_hex,
            } => {
                let ca = read_self_signed_ca(&ca_crt_file, &ca_key_file)?;
                let reader_registration: ReaderRegistration = serde_json::from_reader(reader_auth_file)?;
                let session_transcript_bytes = decode_hex("session transcript", &session_transcript_hex)?;
                let session_transcript = SessionTranscript::try_from_bytes(&session_transcript_bytes)?;
                let common_name = common_name.unwrap_or_else(|| default_reader_common_name(&reader_registration));
                let device_request = block_on(create_reader_device_request(
                    &ca,
                    &common_name,
                    reader_registration,
                    &session_transcript,
                ))?;

                println!("{}", hex::encode(cbor_serialize(&device_request)?));
                Ok(())
            }
            Tsl {
                ca_key_file,
                ca_crt_file,
                common_name,
                file_prefix,
                days,
                force,
            } => {
                let ca = read_self_signed_ca(&ca_crt_file, &ca_key_file)?;
                let key_pair = ca.generate_key_pair(
                    &common_name,
                    OAuthStatusSigning,
                    Self::get_certificate_configuration(days),
                )?;
                write_key_pair(key_pair.certificate(), key_pair.private_key(), &file_prefix, force)?;
                Ok(())
            }
            IssuerCert {
                public_key_file,
                ca_key_file,
                ca_crt_file,
                common_name,
                issuer_auth_file,
                file_prefix,
                days,
                force,
            } => {
                let ca = read_self_signed_ca(&ca_crt_file, &ca_key_file)?;
                let issuer_registration: IssuerRegistration = serde_json::from_reader(issuer_auth_file)?;
                let public_key = read_public_key(&public_key_file)?;

                let certificate = ca.generate_certificate(
                    public_key.contents(),
                    &common_name,
                    issuer_registration,
                    Self::get_certificate_configuration(days),
                )?;
                write_certificate(&certificate, &file_prefix, force)?;
                Ok(())
            }
            ReaderCert {
                public_key_file,
                ca_key_file,
                ca_crt_file,
                common_name,
                reader_auth_file,
                file_prefix,
                days,
                force,
            } => {
                let ca = read_self_signed_ca(&ca_crt_file, &ca_key_file)?;
                let reader_registration: ReaderRegistration = serde_json::from_reader(reader_auth_file)?;
                let public_key = read_public_key(&public_key_file)?;
                let certificate = ca.generate_certificate(
                    public_key.contents(),
                    &common_name,
                    reader_registration,
                    Self::get_certificate_configuration(days),
                )?;
                write_certificate(&certificate, &file_prefix, force)?;
                Ok(())
            }
            TslCert {
                public_key_file,
                ca_key_file,
                ca_crt_file,
                common_name,
                file_prefix,
                days,
                force,
            } => {
                let ca = read_self_signed_ca(&ca_crt_file, &ca_key_file)?;
                let public_key = read_public_key(&public_key_file)?;
                let certificate = ca.generate_certificate(
                    public_key.contents(),
                    &common_name,
                    OAuthStatusSigning,
                    Self::get_certificate_configuration(days),
                )?;
                write_certificate(&certificate, &file_prefix, force)?;
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

fn block_on<F, T>(future: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    let runtime = tokio::runtime::Builder::new_current_thread().build()?;
    runtime.block_on(future)
}

fn decode_hex(label: &str, value: &str) -> Result<Vec<u8>> {
    hex::decode(value).with_context(|| format!("invalid {label} hex"))
}

fn default_reader_common_name(reader_registration: &ReaderRegistration) -> String {
    reader_registration
        .request_origin_base_url
        .host_str()
        .unwrap_or("cert.rp.example.com")
        .to_string()
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
                [namespace, attribute] => (namespace.clone(), attribute.clone()),
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

async fn create_doc_request(
    items_request: ItemsRequest,
    session_transcript: &SessionTranscript,
    key_pair: &crypto::server_keys::KeyPair,
) -> Result<DocRequest> {
    let items_request: ItemsRequestBytes = items_request.into();
    let items_request_for_auth = items_request.clone();
    let reader_auth_keyed = ReaderAuthenticationKeyed::new(session_transcript, &items_request_for_auth);

    let reader_auth = MdocCose::<_, ReaderAuthenticationBytes>::sign(
        &TaggedBytes(CborSeq(reader_auth_keyed)),
        new_certificate_header(key_pair.certificate()),
        key_pair,
        false,
    )
    .await?;

    Ok(DocRequest {
        items_request,
        reader_auth: Some(reader_auth.0.into()),
    })
}

async fn create_reader_device_request(
    ca: &generate::Ca,
    common_name: &str,
    reader_registration: ReaderRegistration,
    session_transcript: &SessionTranscript,
) -> Result<DeviceRequest> {
    let items_requests = items_requests_from_reader_registration(&reader_registration)?;
    let key_pair = ca.generate_key_pair(common_name, reader_registration, CertificateConfiguration::default())?;

    let mut doc_requests = Vec::with_capacity(items_requests.len());
    for items_request in items_requests {
        doc_requests.push(create_doc_request(items_request, session_transcript, &key_pair).await?);
    }

    let doc_requests = VecNonEmpty::try_from(doc_requests)
        .map_err(|_| anyhow!("reader_auth.json did not produce any doc requests"))?;

    Ok(DeviceRequest::from_doc_requests(doc_requests))
}

#[cfg(test)]
mod tests {
    use attestation_data::auth::reader_auth::ReaderRegistration;
    use crypto::server_keys::generate::Ca;
    use crypto::x509::BorrowingCertificateExtension;
    use mdoc::SessionTranscript;
    use mdoc::utils::cose::CoseKey;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use serde_json::json;
    use utils::generator::TimeGenerator;

    use super::create_reader_device_request;
    use super::items_requests_from_reader_registration;

    #[test]
    fn test_items_requests_from_reader_registration_defaults_single_segment_paths_to_doc_type_namespace() {
        let mut reader_registration = ReaderRegistration::new_mock();
        reader_registration.authorized_attributes =
            ReaderRegistration::create_attributes("urn:eudi:pid:nl:1", [["bsn"], ["family_name"]]);

        let items_requests = items_requests_from_reader_registration(&reader_registration).unwrap();
        let items_request = items_requests.first().unwrap();
        let data_elements = items_request.name_spaces.as_ref().get("urn:eudi:pid:nl:1").unwrap();

        assert!(data_elements.as_ref().contains_key("bsn"));
        assert!(data_elements.as_ref().contains_key("family_name"));
    }

    #[test]
    fn test_items_requests_from_reader_registration_match_original_xyz_bank_registration() {
        let reader_registration: ReaderRegistration = serde_json::from_value(json!({
            "purposeStatement": {
                "nl": "Bankrekening openen",
                "en": "Opening bank account"
            },
            "retentionPolicy": {
                "intentToRetain": true,
                "maxDurationInMinutes": 525600
            },
            "sharingPolicy": {
                "intentToShare": false
            },
            "deletionPolicy": {
                "deleteable": true
            },
            "organization": {
                "displayName": {
                    "nl": "XYZ Bank",
                    "en": "XYZ Bank"
                },
                "legalName": {
                    "nl": "XYZ Bank N.V.",
                    "en": "XYZ Bank N.V."
                },
                "description": {
                    "nl": "De toegankelijke bank voor betalen, sparen en beleggen.",
                    "en": "The accessible bank for paying, saving and investing."
                },
                "webUrl": "https://www.xyzbank.nl",
                "privacyPolicyUrl": "https://www.xyzbank.nl/privacy",
                "city": {
                    "nl": "Utrecht",
                    "en": "Utrecht"
                },
                "category": {
                    "nl": "Bank",
                    "en": "Bank"
                },
                "countryCode": "nl",
                "kvk": "12345678"
            },
            "requestOriginBaseUrl": "https://www.xyzbank.nl",
            "authorizedAttributes": {
                "urn:eudi:pid:nl:1": [
                    ["urn:eudi:pid:nl:1", "given_name"],
                    ["urn:eudi:pid:nl:1", "family_name"],
                    ["urn:eudi:pid:nl:1", "birthdate"],
                    ["urn:eudi:pid:nl:1", "bsn"],
                    ["urn:eudi:pid:nl:1.address", "street_address"],
                    ["urn:eudi:pid:nl:1.address", "house_number"],
                    ["urn:eudi:pid:nl:1.address", "postal_code"],
                    ["given_name"],
                    ["family_name"],
                    ["birthdate"],
                    ["bsn"],
                    ["address", "street_address"],
                    ["address", "house_number"],
                    ["address", "postal_code"]
                ],
                "urn:eudi:pid:1": [
                    ["given_name"],
                    ["family_name"],
                    ["birthdate"],
                    ["address", "street_address"],
                    ["address", "house_number"],
                    ["address", "postal_code"]
                ]
            }
        }))
        .unwrap();

        let items_requests = items_requests_from_reader_registration(&reader_registration).unwrap();
        assert_eq!(items_requests.len(), 1);
        assert_eq!(items_requests[0].doc_type, "urn:eudi:pid:nl:1");

        reader_registration
            .verify_requested_attributes(items_requests)
            .expect("generated ItemsRequests should validate against the same ReaderRegistration");
    }

    #[test]
    fn test_create_reader_device_request_signs_verifiable_reader_auth() {
        let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();

        runtime.block_on(async {
            let mut reader_registration = ReaderRegistration::new_mock();
            reader_registration.authorized_attributes = ReaderRegistration::create_attributes(
                "urn:eudi:pid:nl:1",
                vec![vec!["bsn"], vec!["urn:eudi:pid:nl:1.address", "street_address"]],
            );

            let ca = Ca::generate_reader_mock_ca().unwrap();
            let e_reader_key = SigningKey::random(&mut OsRng);
            let cose_key: CoseKey = e_reader_key.verifying_key().try_into().unwrap();
            let session_transcript = SessionTranscript::new_qr(cose_key, None);

            let device_request =
                create_reader_device_request(&ca, "reader.example.com", reader_registration, &session_transcript)
                    .await
                    .unwrap();

            assert_eq!(device_request.doc_requests.len().get(), 1);
            let certificate = device_request
                .doc_requests
                .first()
                .verify(&session_transcript, &TimeGenerator, &[ca.to_trust_anchor()])
                .unwrap()
                .unwrap();

            let parsed_registration = ReaderRegistration::from_certificate(&certificate).unwrap().unwrap();
            assert_eq!(
                parsed_registration.request_origin_base_url.as_str(),
                "https://example.com/"
            );
        });
    }
}
