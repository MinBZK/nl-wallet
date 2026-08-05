use std::cmp::Ordering;
use std::ops::Add;
use std::ops::Sub;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use assert_cmd::prelude::*;
use assert_fs::TempDir;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::*;
use attestation_data::auth::issuer_auth::IssuerRegistration;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateUsage;
use crypto::x509::DistinguishedName;
use crypto::x509::SubjectAltNameUri;
use crypto::x509::crl::extract_crl_distribution_points;
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::Generate;
use p256::pkcs8::DecodePrivateKey;
use p256::pkcs8::EncodePublicKey;
use p256::pkcs8::spki::DynAssociatedAlgorithmIdentifier;
use p256::pkcs8::spki::ObjectIdentifier;
use pem::EncodeConfig;
use pem::LineEnding;
use pem::Pem;
use predicates::prelude::*;
use predicates::str::RegexPredicate;
use predicates::str::StartsWithPredicate;
use time::Duration;
use time::OffsetDateTime;
use url::Url;
use x509_parser::extensions::GeneralName;
use x509_parser::num_bigint::BigUint;
use x509_parser::oid_registry::OID_KEY_TYPE_EC_PUBLIC_KEY;
use x509_parser::parse_x509_crl;

trait RangeCompare<Offset> {
    /// Compare [`self`] to the range of [`other`] +/- the [`offset`].
    fn cmp_range(&self, other: &Self, offset: Offset) -> Ordering;
}

impl<T, R> RangeCompare<R> for T
where
    T: Add<R, Output = Self>,
    T: Sub<R, Output = Self>,
    T: Ord,
    T: Copy,
    R: Copy,
{
    // This comparison is performed inclusive on the bounds.
    fn cmp_range(&self, other: &Self, offset: R) -> Ordering {
        if self.cmp(&(*other - offset)) == Ordering::Less {
            return Ordering::Less;
        }
        if self.cmp(&(*other + offset)) == Ordering::Greater {
            return Ordering::Greater;
        }
        Ordering::Equal
    }
}

fn predicate_successfully_generated_key_pair(crt: &Path, key: &Path) -> Result<RegexPredicate> {
    let result = predicate::str::is_match(format!(
        "Certificate stored in '{}'\nKey stored in '{}'",
        crt.display(),
        key.display(),
    ))?;
    Ok(result)
}

fn predicate_successfully_generated_certificate(crt: &Path) -> Result<RegexPredicate> {
    let result = predicate::str::is_match(format!("Certificate stored in '{}'", crt.display(),))?;
    Ok(result)
}

fn predicate_successfully_generated_crl(pem_crl: &Path) -> Result<RegexPredicate> {
    let result = predicate::str::is_match(format!("CRL stored in '{}'", pem_crl.display()))?;
    Ok(result)
}

fn predicate_invalid_serial_number() -> predicates::str::ContainsPredicate {
    predicate::str::contains("invalid hex-encoded serial number")
}

fn predicate_file_already_exists(path: &Path) -> Result<RegexPredicate> {
    let result = predicate::str::is_match(format!("Error: Target file '{}' already exists\n", path.display()))?;
    Ok(result)
}

fn predicate_not_a_natural_or_legal_person() -> Result<RegexPredicate> {
    let result =
        predicate::str::is_match("Error: Illegal subject name, specify either for a legal or natural person\n")?;
    Ok(result)
}

fn predicate_missing_issuer_json_file(path: &Path) -> StartsWithPredicate {
    predicate::str::starts_with(format!(
        "error: Invalid value for --issuer-auth-file <ISSUER_AUTH_FILE>: Could not open \"{}\": No such file or \
         directory",
        path.display()
    ))
}

fn predicate_missing_crt_file(path: &Path) -> StartsWithPredicate {
    predicate::str::starts_with(format!(
        r#"error: Invalid value for --ca-crt-file <CA_CRT_FILE>: Could not open "{}": No such file or directory"#,
        path.display()
    ))
}

fn predicate_missing_key_file(path: &Path) -> StartsWithPredicate {
    predicate::str::starts_with(format!(
        r#"error: Invalid value for --ca-key-file <CA_KEY_FILE>: Could not open "{}": No such file or directory"#,
        path.display()
    ))
}

fn predicate_missing_public_key_file(path: &Path) -> StartsWithPredicate {
    predicate::str::starts_with(format!(
        r#"error: Invalid value for --public-key-file <PUBLIC_KEY_FILE>: Could not open "{}": No such file or directory"#,
        path.display()
    ))
}

fn assert_generated_key(key_file: &ChildPath) -> Result<()> {
    // Read key and verify algorithm
    SigningKey::read_pkcs8_pem_file(key_file)?
        .algorithm_identifier()?
        .assert_algorithm_oid(ObjectIdentifier::new_unwrap(&OID_KEY_TYPE_EC_PUBLIC_KEY.to_id_string()))?;

    Ok(())
}

#[expect(clippy::too_many_arguments, reason = "test function")]
fn assert_generated_certificate(
    crt_file: &ChildPath,
    expected_issuer_dn: &DistinguishedName,
    expected_subject_dn: &DistinguishedName,
    expected_san_uri: Option<&SubjectAltNameUri>,
    start: OffsetDateTime,
    end: OffsetDateTime,
    usage: Option<CertificateUsage>,
) -> Result<()> {
    // Read certificate and verify PEM label
    let crt_pem_bytes = std::fs::read(crt_file)?;
    let (_, crt_pem) = x509_parser::pem::parse_x509_pem(&crt_pem_bytes)?;
    assert_eq!(crt_pem.label, "CERTIFICATE");
    let crt = crt_pem.parse_x509()?;

    let issuer_dn = DistinguishedName::try_from(crt.issuer())?;
    assert_eq!(&issuer_dn, expected_issuer_dn);

    let subject_dn = DistinguishedName::try_from(crt.subject())?;
    assert_eq!(&subject_dn, expected_subject_dn);

    // verify SAN URI
    if let Some(expected_san_uri) = expected_san_uri {
        itertools::assert_equal(
            crt.subject_alternative_name()
                .unwrap()
                .unwrap()
                .value
                .general_names
                .iter()
                .map(|gn| match gn {
                    GeneralName::URI(uri) => uri.to_string(),
                    _ => panic!("SAN is not a URI"),
                }),
            vec![expected_san_uri.as_ref().to_string()],
        );
    } else {
        assert!(crt.subject_alternative_name().unwrap().is_none());
    }

    // verify validity times with minute accuracy
    let not_before = crt.validity().not_before.to_datetime();
    assert_eq!(not_before.cmp_range(&start, Duration::minutes(1)), Ordering::Equal);
    let not_after = crt.validity().not_after.to_datetime();
    assert_eq!(not_after.cmp_range(&end, Duration::minutes(1)), Ordering::Equal);

    // verify usage
    if let Some(usage) = usage {
        assert_eq!(CertificateUsage::from_certificate(&crt)?, usage);
    }

    Ok(())
}

/// Verify the CRL Distribution Point URIs from a generated certificate's CDP extension against the expected CRL
/// Distribution Point URIs.
fn assert_generated_certificate_crl_distribution_points(crt_file: &ChildPath, expected: &[Url]) -> Result<()> {
    let crt_pem_bytes = std::fs::read(crt_file)?;
    let (_, crt_pem) = x509_parser::pem::parse_x509_pem(&crt_pem_bytes)?;
    let cert = BorrowingCertificate::from_der(crt_pem.contents)?;

    let uris = extract_crl_distribution_points(&cert)
        .map(|urls| urls.into_inner())
        .unwrap_or_default();

    let expected: Vec<String> = expected.iter().map(ToString::to_string).collect();
    assert_eq!(uris, expected);

    Ok(())
}

/// Verify a generated CRL's issuer, validity window, `crlNumber` and revoked serial numbers (as raw bytes, in the
/// order they appear on the CRL).
fn assert_generated_crl(
    pem_crl_file: &ChildPath,
    expected_issuer_dn: &DistinguishedName,
    start: OffsetDateTime,
    end: OffsetDateTime,
    expected_revoked_serials: &[Vec<u8>],
) -> Result<()> {
    // Read CRL and verify PEM label
    let crl_pem_bytes = std::fs::read(pem_crl_file)?;
    let (_, crl_pem) = x509_parser::pem::parse_x509_pem(&crl_pem_bytes)?;
    assert_eq!(crl_pem.label, "X509 CRL");
    let (_, crl) = parse_x509_crl(&crl_pem.contents)?;

    let issuer_dn = DistinguishedName::try_from(crl.issuer())?;
    assert_eq!(&issuer_dn, expected_issuer_dn);

    // verify thisUpdate/nextUpdate with minute accuracy
    let this_update = crl.last_update().to_datetime();
    assert_eq!(this_update.cmp_range(&start, Duration::minutes(1)), Ordering::Equal);
    let next_update = crl.next_update().expect("CRL should have a nextUpdate").to_datetime();
    assert_eq!(next_update.cmp_range(&end, Duration::minutes(1)), Ordering::Equal);

    // The initial crlNumber starts the sequence at thisUpdate. Regeneration tests separately verify that the existing
    // DER file advances the sequence.
    let crl_number = crl.crl_number().expect("CRL should have a crlNumber");
    let expected_crl_number = BigUint::from(u64::try_from(this_update.unix_timestamp())?);
    assert_eq!(crl_number, &expected_crl_number);

    // verify revoked serial numbers, in order
    let revoked_serials: Vec<Vec<u8>> = crl
        .iter_revoked_certificates()
        .map(|revoked| revoked.raw_serial().to_vec())
        .collect();
    assert_eq!(revoked_serials, expected_revoked_serials);

    Ok(())
}

trait CommandExtension {
    fn generate_ca(&mut self, file_prefix: &Path) -> &mut Self;
    fn generate_issuer_kp(
        &mut self,
        ca_crt: &Path,
        ca_key: &Path,
        issuer_auth_json: &Path,
        file_prefix: &Path,
    ) -> &mut Self;
    fn generate_wrpac_kp(&mut self, ca_crt: &Path, ca_key: &Path, file_prefix: &Path) -> &mut Self;
    fn generate_tsl_kp(&mut self, ca_crt: &Path, ca_key: &Path, file_prefix: &Path) -> &mut Self;
    fn generate_issuer_cert(
        &mut self,
        pk: &Path,
        ca_crt: &Path,
        ca_key: &Path,
        issuer_auth_json: &Path,
        file_prefix: &Path,
    ) -> &mut Self;
    fn generate_wrpac_cert(&mut self, pk: &Path, ca_crt: &Path, ca_key: &Path, file_prefix: &Path) -> &mut Self;
    fn generate_tsl_cert(&mut self, pk: &Path, ca_crt: &Path, ca_key: &Path, file_prefix: &Path) -> &mut Self;
    fn generate_crl(&mut self, ca_crt: &Path, ca_key: &Path, file_prefix: &Path, days: &str) -> &mut Self;

    fn generate_for_legal_person(&mut self, organization_name: &str, organization_identifer: &str) -> &mut Self;

    fn generate_for_natural_person(&mut self, serial_number: &str, surname: &str, given_name: &str) -> &mut Self;
}

impl CommandExtension for Command {
    fn generate_ca(&mut self, file_prefix: &Path) -> &mut Self {
        self.arg("ca")
            .arg("--common-name")
            .arg("CA")
            .arg("--file-prefix")
            .arg(file_prefix)
    }

    fn generate_issuer_kp(
        &mut self,
        ca_crt: &Path,
        ca_key: &Path,
        issuer_auth_json: &Path,
        file_prefix: &Path,
    ) -> &mut Self {
        self.arg("cert")
            .arg("--type")
            .arg("issuer")
            .arg("--ca-key-file")
            .arg(ca_key)
            .arg("--ca-crt-file")
            .arg(ca_crt)
            .arg("--common-name")
            .arg("Test Issuer")
            .arg("--file-prefix")
            .arg(file_prefix)
            .arg("--issuer-auth-file")
            .arg(issuer_auth_json)
    }

    fn generate_wrpac_kp(&mut self, ca_crt: &Path, ca_key: &Path, file_prefix: &Path) -> &mut Self {
        self.arg("cert")
            .arg("--type")
            .arg("wrpac")
            .arg("--ca-key-file")
            .arg(ca_key)
            .arg("--ca-crt-file")
            .arg(ca_crt)
            .arg("--common-name")
            .arg("Test WRPAC")
            .arg("--file-prefix")
            .arg(file_prefix)
    }

    fn generate_tsl_kp(&mut self, ca_crt: &Path, ca_key: &Path, file_prefix: &Path) -> &mut Self {
        self.arg("cert")
            .arg("--type")
            .arg("tsl")
            .arg("--ca-key-file")
            .arg(ca_key)
            .arg("--ca-crt-file")
            .arg(ca_crt)
            .arg("--common-name")
            .arg("Test TSL")
            .arg("--file-prefix")
            .arg(file_prefix)
    }

    fn generate_issuer_cert(
        &mut self,
        pk: &Path,
        ca_crt: &Path,
        ca_key: &Path,
        issuer_auth_json: &Path,
        file_prefix: &Path,
    ) -> &mut Self {
        self.arg("cert-pub")
            .arg("--type")
            .arg("issuer")
            .arg("--public-key-file")
            .arg(pk)
            .arg("--ca-key-file")
            .arg(ca_key)
            .arg("--ca-crt-file")
            .arg(ca_crt)
            .arg("--common-name")
            .arg("Test Issuer")
            .arg("--file-prefix")
            .arg(file_prefix)
            .arg("--issuer-auth-file")
            .arg(issuer_auth_json)
    }

    fn generate_wrpac_cert(&mut self, pk: &Path, ca_crt: &Path, ca_key: &Path, file_prefix: &Path) -> &mut Self {
        self.arg("cert-pub")
            .arg("--type")
            .arg("wrpac")
            .arg("--public-key-file")
            .arg(pk)
            .arg("--ca-key-file")
            .arg(ca_key)
            .arg("--ca-crt-file")
            .arg(ca_crt)
            .arg("--common-name")
            .arg("Test WRPAC")
            .arg("--file-prefix")
            .arg(file_prefix)
    }

    fn generate_tsl_cert(&mut self, pk: &Path, ca_crt: &Path, ca_key: &Path, file_prefix: &Path) -> &mut Self {
        self.arg("cert-pub")
            .arg("--type")
            .arg("tsl")
            .arg("--public-key-file")
            .arg(pk)
            .arg("--ca-key-file")
            .arg(ca_key)
            .arg("--ca-crt-file")
            .arg(ca_crt)
            .arg("--common-name")
            .arg("Test TSL")
            .arg("--file-prefix")
            .arg(file_prefix)
    }

    fn generate_crl(&mut self, ca_crt: &Path, ca_key: &Path, file_prefix: &Path, days: &str) -> &mut Self {
        self.arg("crl")
            .arg("--ca-key-file")
            .arg(ca_key)
            .arg("--ca-crt-file")
            .arg(ca_crt)
            .arg("--file-prefix")
            .arg(file_prefix)
            .arg("--days")
            .arg(days)
    }

    fn generate_for_legal_person(&mut self, organization_name: &str, organization_identifer: &str) -> &mut Self {
        self.arg("--organization-name")
            .arg(organization_name)
            .arg("--organization-id")
            .arg(organization_identifer)
    }

    fn generate_for_natural_person(&mut self, serial_number: &str, surname: &str, given_name: &str) -> &mut Self {
        self.arg("--serial-number")
            .arg(serial_number)
            .arg("--surname")
            .arg(surname)
            .arg("--given-name")
            .arg(given_name)
    }
}

fn keypair_paths(temp: &TempDir, prefix: &str) -> (ChildPath, ChildPath, ChildPath) {
    (
        temp.child(prefix),
        temp.child(format!("{prefix}.crt.pem")),
        temp.child(format!("{prefix}.key.pem")),
    )
}

fn public_key_path(temp: &TempDir, prefix: &str) -> ChildPath {
    temp.child(format!("{prefix}.pk.pem"))
}

fn crl_path(temp: &TempDir, prefix: &str) -> ChildPath {
    temp.child(format!("{prefix}.crl.pem"))
}

fn generate_public_key(path: &ChildPath) {
    let signing_key = SigningKey::generate();
    let public_key = signing_key.verifying_key();
    let der = public_key.to_public_key_der().unwrap();
    let pem = Pem::new("PUBLIC KEY", der.to_vec());
    std::fs::write(
        path,
        pem::encode_config(&pem, EncodeConfig::new().set_line_ending(LineEnding::LF)),
    )
    .unwrap();
}

const DEFAULT_LIFETIME: Duration = Duration::days(365);
const DEFAULT_CRL_LIFETIME: Duration = Duration::days(90);

#[test]
fn happy_flow_with_default_lifetime() -> Result<()> {
    let temp = TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");

    // Generate ca and assert success and stderr output
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_ca(&ca_prefix)
        .assert()
        .success()
        .stderr(predicate_successfully_generated_key_pair(&ca_crt, &ca_key)?);

    // Assert generated ca files
    let ca_dn = DistinguishedName::new("CA".to_string(), "NL".to_string());
    assert_generated_key(&ca_key)?;
    assert_generated_certificate(
        &ca_crt,
        &ca_dn,
        &ca_dn,
        None,
        OffsetDateTime::now_utc(),
        OffsetDateTime::now_utc() + DEFAULT_LIFETIME,
        None,
    )?;

    // Generate issuer key pair
    {
        let (mdl_prefix, mdl_crt, mdl_key) = keypair_paths(&temp, "test-mdl-kp");
        let issuer_auth_json = temp.child("test-issuer-auth.json");

        // Generate issuer registration JSON input file
        issuer_auth_json.write_str(&serde_json::to_string(&IssuerRegistration::new_mock())?)?;

        // Execute command and assert success and stderr output
        Command::new(assert_cmd::cargo::cargo_bin!())
            .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
            .generate_for_natural_person("123", "Doe", "John")
            .assert()
            .success()
            .stderr(predicate_successfully_generated_key_pair(&mdl_crt, &mdl_key)?);

        // Assert generated issuer files
        assert_generated_key(&mdl_key)?;
        assert_generated_certificate(
            &mdl_crt,
            &ca_dn,
            &DistinguishedName::new_natural_person(
                "Test Issuer".to_string(),
                "NL".to_string(),
                "123".to_string(),
                "Doe".to_string(),
                "John".to_string(),
            ),
            None,
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + DEFAULT_LIFETIME,
            Some(CertificateUsage::Mdl),
        )?;
    }

    // Generate issuer certificate
    {
        let (mdl_prefix, mdl_crt, _) = keypair_paths(&temp, "test-mdl-crt");
        let issuer_auth_json = temp.child("test-issuer-auth.json");

        // Generate issuer registration JSON input file
        issuer_auth_json.write_str(&serde_json::to_string(&IssuerRegistration::new_mock())?)?;

        let public_key_path = public_key_path(&temp, "test-mdl-crt");
        generate_public_key(&public_key_path);

        // Execute command and assert success and stderr output
        Command::new(assert_cmd::cargo::cargo_bin!())
            .generate_issuer_cert(&public_key_path, &ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
            .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
            .assert()
            .success()
            .stderr(predicate_successfully_generated_certificate(&mdl_crt)?);

        // Assert generated issuer certificate
        assert_generated_certificate(
            &mdl_crt,
            &ca_dn,
            &DistinguishedName::new_legal_person(
                "Test Issuer".to_string(),
                "NL".to_string(),
                "Test B.V.".to_string(),
                "NTRNL-00000002".to_string(),
            ),
            None,
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + DEFAULT_LIFETIME,
            Some(CertificateUsage::Mdl),
        )?;
    }

    // Generate WRPAC key pair
    {
        let (rp_auth_prefix, rp_auth_crt, rp_auth_key) = keypair_paths(&temp, "test-wrpac-auth-kp");

        // Execute command and assert success and stderr output
        Command::new(assert_cmd::cargo::cargo_bin!())
            .generate_wrpac_kp(&ca_crt, &ca_key, &rp_auth_prefix)
            .generate_for_natural_person("123", "Doe", "John")
            .assert()
            .success()
            .stderr(predicate_successfully_generated_key_pair(&rp_auth_crt, &rp_auth_key)?);

        // Assert generated WRPAC files
        assert_generated_key(&rp_auth_key)?;
        assert_generated_certificate(
            &rp_auth_crt,
            &ca_dn,
            &DistinguishedName::new_natural_person(
                "Test WRPAC".to_string(),
                "NL".to_string(),
                "123".to_string(),
                "Doe".to_string(),
                "John".to_string(),
            ),
            None,
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + DEFAULT_LIFETIME,
            None,
        )?;
    }

    // Generate WRPAC certificate
    {
        let (rp_auth_prefix, rp_auth_crt, _) = keypair_paths(&temp, "test-wrpac-auth-crt");

        let public_key_path = public_key_path(&temp, "test-wrpac-auth-crt");
        generate_public_key(&public_key_path);

        // Execute command and assert success and stderr output
        Command::new(assert_cmd::cargo::cargo_bin!())
            .generate_wrpac_cert(&public_key_path, &ca_crt, &ca_key, &rp_auth_prefix)
            .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
            .assert()
            .success()
            .stderr(predicate_successfully_generated_certificate(&rp_auth_crt)?);

        // Assert generated WRPAC certificate
        assert_generated_certificate(
            &rp_auth_crt,
            &ca_dn,
            &DistinguishedName::new_legal_person(
                "Test WRPAC".to_string(),
                "NL".to_string(),
                "Test B.V.".to_string(),
                "NTRNL-00000002".to_string(),
            ),
            None,
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + DEFAULT_LIFETIME,
            None,
        )?;
    }

    // Generate tsl key pair
    {
        let (tsl_prefix, tsl_crt, tsl_key) = keypair_paths(&temp, "test-tsl-kp");

        // Execute command and assert success and stderr output
        Command::new(assert_cmd::cargo::cargo_bin!())
            .generate_tsl_kp(&ca_crt, &ca_key, &tsl_prefix)
            .generate_for_natural_person("123", "Doe", "John")
            .assert()
            .success()
            .stderr(predicate_successfully_generated_key_pair(&tsl_crt, &tsl_key)?);

        // Assert generated TSL files
        assert_generated_key(&tsl_key)?;
        assert_generated_certificate(
            &tsl_crt,
            &ca_dn,
            &DistinguishedName::new_natural_person(
                "Test TSL".to_string(),
                "NL".to_string(),
                "123".to_string(),
                "Doe".to_string(),
                "John".to_string(),
            ),
            None,
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + DEFAULT_LIFETIME,
            Some(CertificateUsage::OAuthStatusSigning),
        )?;
    }

    // Generate tsl certificate
    {
        let (tsl_prefix, tsl_crt, _) = keypair_paths(&temp, "test-tsl-crt");

        let public_key_path = public_key_path(&temp, "test-tsl-crt");
        generate_public_key(&public_key_path);

        // Execute command and assert success and stderr output
        Command::new(assert_cmd::cargo::cargo_bin!())
            .generate_tsl_cert(&public_key_path, &ca_crt, &ca_key, &tsl_prefix)
            .generate_for_legal_person("Test GmbH", "NTRNL-00000002")
            .arg("--country-name")
            .arg("DE")
            .assert()
            .success()
            .stderr(predicate_successfully_generated_certificate(&tsl_crt)?);

        // Assert generated TSL certificate
        assert_generated_certificate(
            &tsl_crt,
            &ca_dn,
            &DistinguishedName::new_legal_person(
                "Test TSL".to_string(),
                "DE".to_string(),
                "Test GmbH".to_string(),
                "NTRNL-00000002".to_string(),
            ),
            None,
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + DEFAULT_LIFETIME,
            Some(CertificateUsage::OAuthStatusSigning),
        )?;
    }

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}

#[test]
fn happy_flow_with_custom_lifetime() -> Result<()> {
    let temp = TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");

    // Generate ca and assert success and stderr output
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_ca(&ca_prefix)
        .arg("--days")
        .arg("586")
        .assert()
        .success()
        .stderr(predicate_successfully_generated_key_pair(&ca_crt, &ca_key)?);

    // Assert generated ca files
    let ca_dn = DistinguishedName::new("CA".to_string(), "NL".to_string());
    assert_generated_key(&ca_key)?;
    assert_generated_certificate(
        &ca_crt,
        &ca_dn,
        &ca_dn,
        None,
        OffsetDateTime::now_utc(),
        OffsetDateTime::now_utc() + Duration::days(586),
        None,
    )?;

    // Generate tsl key pair
    {
        let (tsl_prefix, tsl_crt, tsl_key) = keypair_paths(&temp, "test-tsl-kp");

        // Execute command and assert success and stderr output
        Command::new(assert_cmd::cargo::cargo_bin!())
            .generate_tsl_kp(&ca_crt, &ca_key, &tsl_prefix)
            .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
            .arg("--days")
            .arg("7")
            .assert()
            .success()
            .stderr(predicate_successfully_generated_key_pair(&tsl_crt, &tsl_key)?);

        // Assert generated certificate files
        assert_generated_key(&tsl_key)?;
        assert_generated_certificate(
            &tsl_crt,
            &ca_dn,
            &DistinguishedName::new_legal_person(
                "Test TSL".to_string(),
                "NL".to_string(),
                "Test B.V.".to_string(),
                "NTRNL-00000002".to_string(),
            ),
            None,
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + Duration::days(7),
            Some(CertificateUsage::OAuthStatusSigning),
        )?;
    }

    // Generate tsl certificate
    {
        let (tsl_prefix, tsl_crt, _) = keypair_paths(&temp, "test-tsl-crt");

        let public_key_path = public_key_path(&temp, "test-tsl-crt");
        generate_public_key(&public_key_path);

        // Execute command and assert success and stderr output
        Command::new(assert_cmd::cargo::cargo_bin!())
            .generate_tsl_cert(&public_key_path, &ca_crt, &ca_key, &tsl_prefix)
            .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
            .arg("--days")
            .arg("7")
            .assert()
            .success()
            .stderr(predicate_successfully_generated_certificate(&tsl_crt)?);

        // Assert generated certificate
        assert_generated_certificate(
            &tsl_crt,
            &ca_dn,
            &DistinguishedName::new_legal_person(
                "Test TSL".to_string(),
                "NL".to_string(),
                "Test B.V.".to_string(),
                "NTRNL-00000002".to_string(),
            ),
            None,
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + Duration::days(7),
            Some(CertificateUsage::OAuthStatusSigning),
        )?;
    }

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}

#[test]
fn happy_flow_with_san() -> Result<()> {
    let temp = TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");

    // Generate ca and assert success and stderr output
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_ca(&ca_prefix)
        .assert()
        .success()
        .stderr(predicate_successfully_generated_key_pair(&ca_crt, &ca_key)?);

    // Assert generated ca files
    let ca_dn = DistinguishedName::new("CA".to_string(), "NL".to_string());
    assert_generated_key(&ca_key)?;
    assert_generated_certificate(
        &ca_crt,
        &ca_dn,
        &ca_dn,
        None,
        OffsetDateTime::now_utc(),
        OffsetDateTime::now_utc() + DEFAULT_LIFETIME,
        None,
    )?;

    // Generate tsl key pair
    {
        let (tsl_prefix, tsl_crt, tsl_key) = keypair_paths(&temp, "test-tsl-kp");

        // Execute command and assert success and stderr output
        Command::new(assert_cmd::cargo::cargo_bin!())
            .generate_tsl_kp(&ca_crt, &ca_key, &tsl_prefix)
            .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
            .arg("--san-uri")
            .arg("https://tsl.example.com")
            .assert()
            .success()
            .stderr(predicate_successfully_generated_key_pair(&tsl_crt, &tsl_key)?);

        // Assert generated cert files
        assert_generated_key(&tsl_key)?;
        assert_generated_certificate(
            &tsl_crt,
            &ca_dn,
            &DistinguishedName::new_legal_person(
                "Test TSL".to_string(),
                "NL".to_string(),
                "Test B.V.".to_string(),
                "NTRNL-00000002".to_string(),
            ),
            Some(&"https://tsl.example.com".parse().unwrap()),
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + DEFAULT_LIFETIME,
            Some(CertificateUsage::OAuthStatusSigning),
        )?;
    }

    // Generate tsl certificate
    {
        let (tsl_prefix, tsl_crt, _) = keypair_paths(&temp, "test-tsl-crt");

        let public_key_path = public_key_path(&temp, "test-tsl-crt");
        generate_public_key(&public_key_path);

        // Execute command and assert success and stderr output
        Command::new(assert_cmd::cargo::cargo_bin!())
            .generate_tsl_cert(&public_key_path, &ca_crt, &ca_key, &tsl_prefix)
            .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
            .arg("--san-uri")
            .arg("https://tsl.example.com")
            .assert()
            .success()
            .stderr(predicate_successfully_generated_certificate(&tsl_crt)?);

        // Assert generated certificate
        assert_generated_certificate(
            &tsl_crt,
            &ca_dn,
            &DistinguishedName::new_legal_person(
                "Test TSL".to_string(),
                "NL".to_string(),
                "Test B.V.".to_string(),
                "NTRNL-00000002".to_string(),
            ),
            Some(&"https://tsl.example.com".parse().unwrap()),
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + DEFAULT_LIFETIME,
            Some(CertificateUsage::OAuthStatusSigning),
        )?;
    }

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}

#[test]
fn happy_flow_with_crl_distribution_points() -> Result<()> {
    let temp = TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");

    // Generate ca and assert success
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_ca(&ca_prefix)
        .assert()
        .success()
        .stderr(predicate_successfully_generated_key_pair(&ca_crt, &ca_key)?);

    let crl_distribution_points: Vec<Url> = vec![
        "http://crl1.example.com/wrpac.crl".parse().unwrap(),
        "http://crl2.example.com/wrpac.crl".parse().unwrap(),
    ];

    // Generate WRPAC key pair with multiple CDPs
    {
        let (rp_auth_prefix, rp_auth_crt, rp_auth_key) = keypair_paths(&temp, "test-wrpac-cdp-kp");

        Command::new(assert_cmd::cargo::cargo_bin!())
            .generate_wrpac_kp(&ca_crt, &ca_key, &rp_auth_prefix)
            .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
            .arg("--crl-distribution-point")
            .args(crl_distribution_points.iter().map(Url::as_str))
            .assert()
            .success()
            .stderr(predicate_successfully_generated_key_pair(&rp_auth_crt, &rp_auth_key)?);

        assert_generated_key(&rp_auth_key)?;
        assert_generated_certificate_crl_distribution_points(&rp_auth_crt, &crl_distribution_points)?;
    }

    // Generate WRPAC certificate with multiple CDPs
    {
        let (rp_auth_prefix, rp_auth_crt, _) = keypair_paths(&temp, "test-wrpac-cdp-crt");

        let public_key_path = public_key_path(&temp, "test-wrpac-cdp-crt");
        generate_public_key(&public_key_path);

        Command::new(assert_cmd::cargo::cargo_bin!())
            .generate_wrpac_cert(&public_key_path, &ca_crt, &ca_key, &rp_auth_prefix)
            .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
            .arg("--crl-distribution-point")
            .args(crl_distribution_points.iter().map(Url::as_str))
            .assert()
            .success()
            .stderr(predicate_successfully_generated_certificate(&rp_auth_crt)?);

        assert_generated_certificate_crl_distribution_points(&rp_auth_crt, &crl_distribution_points)?;
    }

    // Generate a WRPAC certificate without the flag: no CDP extension should be present
    {
        let (rp_auth_prefix, rp_auth_crt, _) = keypair_paths(&temp, "test-wrpac-no-cdp-crt");

        let public_key_path = public_key_path(&temp, "test-wrpac-no-cdp-crt");
        generate_public_key(&public_key_path);

        Command::new(assert_cmd::cargo::cargo_bin!())
            .generate_wrpac_cert(&public_key_path, &ca_crt, &ca_key, &rp_auth_prefix)
            .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
            .assert()
            .success()
            .stderr(predicate_successfully_generated_certificate(&rp_auth_crt)?);

        assert_generated_certificate_crl_distribution_points(&rp_auth_crt, &[])?;
    }

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}

#[test]
fn happy_flow_crl_empty() -> Result<()> {
    let temp = TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");

    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_ca(&ca_prefix)
        .assert()
        .success()
        .stderr(predicate_successfully_generated_key_pair(&ca_crt, &ca_key)?);

    let ca_dn = DistinguishedName::new("CA".to_string(), "NL".to_string());
    let crl_prefix = temp.child("test-crl");
    let crl = crl_path(&temp, "test-crl");

    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_crl(&ca_crt, &ca_key, &crl_prefix, "7")
        .assert()
        .success()
        .stderr(predicate_successfully_generated_crl(&crl)?);

    assert_generated_crl(
        &crl,
        &ca_dn,
        OffsetDateTime::now_utc(),
        OffsetDateTime::now_utc() + Duration::days(7),
        &[],
    )?;
    assert!(!temp.child("test-crl.crl.der").exists());

    temp.close()?;

    Ok(())
}

#[test]
fn happy_flow_crl_with_revoked_certificates() -> Result<()> {
    let temp = TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");

    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_ca(&ca_prefix)
        .assert()
        .success()
        .stderr(predicate_successfully_generated_key_pair(&ca_crt, &ca_key)?);

    let ca_dn = DistinguishedName::new("CA".to_string(), "NL".to_string());
    let crl_prefix = temp.child("test-crl");
    let crl = crl_path(&temp, "test-crl");

    // Serial numbers are free-form input to the `crl` command (it never looks at an actual
    // certificate), so cover both accepted formats with literal values: colon-separated hex, as
    // `openssl x509 -text` prints it, and plain hex, as `-noout -serial` prints it.
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_crl(&ca_crt, &ca_key, &crl_prefix, "90")
        .arg("--serial-number")
        .arg("1a:2b:3c:4d")
        .arg("01020304")
        .assert()
        .success()
        .stderr(predicate_successfully_generated_crl(&crl)?);

    assert_generated_crl(
        &crl,
        &ca_dn,
        OffsetDateTime::now_utc(),
        OffsetDateTime::now_utc() + DEFAULT_CRL_LIFETIME,
        &[vec![0x1a, 0x2b, 0x3c, 0x4d], vec![0x01, 0x02, 0x03, 0x04]],
    )?;

    temp.close()?;

    Ok(())
}

#[test]
fn crl_generation_with_invalid_serial_number() -> Result<()> {
    let temp = TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");

    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_ca(&ca_prefix)
        .assert()
        .success();

    let crl_prefix = temp.child("test-crl");

    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_crl(&ca_crt, &ca_key, &crl_prefix, "90")
        .arg("--serial-number")
        .arg("not-hex")
        .assert()
        .failure()
        .stderr(predicate_invalid_serial_number());

    temp.close()?;

    Ok(())
}

#[test]
fn regenerating_crl() -> Result<()> {
    let temp = TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");

    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_ca(&ca_prefix)
        .assert()
        .success();

    let crl_prefix = temp.child("test-crl");
    let crl = crl_path(&temp, "test-crl");

    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_crl(&ca_crt, &ca_key, &crl_prefix, "90")
        .assert()
        .success();
    let first_crl_pem_bytes = std::fs::read(&crl)?;
    let (_, first_crl_pem) = x509_parser::pem::parse_x509_pem(&first_crl_pem_bytes)?;
    let (_, first_crl) = parse_x509_crl(&first_crl_pem.contents)?;
    let first_crl_number = first_crl.crl_number().expect("CRL should have a crlNumber").clone();

    // Re-generating the CRL should fail without --force
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_crl(&ca_crt, &ca_key, &crl_prefix, "90")
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&crl)?);

    // Re-generating the CRL should succeed with --force
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_crl(&ca_crt, &ca_key, &crl_prefix, "90")
        .arg("--force")
        .assert()
        .success();
    let second_crl_pem_bytes = std::fs::read(&crl)?;
    let (_, second_crl_pem) = x509_parser::pem::parse_x509_pem(&second_crl_pem_bytes)?;
    let (_, second_crl) = parse_x509_crl(&second_crl_pem.contents)?;
    let second_crl_number = second_crl.crl_number().expect("CRL should have a crlNumber");

    assert!(second_crl_number > &first_crl_number);

    temp.close()?;

    Ok(())
}

#[test]
fn not_a_natural_or_legal_person() -> Result<()> {
    let temp = TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");

    // Generate ca and assert success and stderr output
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_ca(&ca_prefix)
        .assert()
        .success()
        .stderr(predicate_successfully_generated_key_pair(&ca_crt, &ca_key)?);

    // Assert generated ca files
    let ca_dn = DistinguishedName::new("CA".to_string(), "NL".to_string());
    assert_generated_key(&ca_key)?;
    assert_generated_certificate(
        &ca_crt,
        &ca_dn,
        &ca_dn,
        None,
        OffsetDateTime::now_utc(),
        OffsetDateTime::now_utc() + DEFAULT_LIFETIME,
        None,
    )?;

    // Generate tsl key pair
    {
        let (tsl_prefix, _, _) = keypair_paths(&temp, "test-tsl-kp");

        // Execute command and assert success and stderr output
        Command::new(assert_cmd::cargo::cargo_bin!())
            .generate_tsl_kp(&ca_crt, &ca_key, &tsl_prefix)
            .arg("--serial-number")
            .arg("1234")
            .assert()
            .failure()
            .stderr(predicate_not_a_natural_or_legal_person()?);
    }

    // Generate tsl certificate
    {
        let (tsl_prefix, _, _) = keypair_paths(&temp, "test-tsl-crt");

        let public_key_path = public_key_path(&temp, "test-tsl-crt");
        generate_public_key(&public_key_path);

        // Execute command and assert success and stderr output
        Command::new(assert_cmd::cargo::cargo_bin!())
            .generate_tsl_cert(&public_key_path, &ca_crt, &ca_key, &tsl_prefix)
            .arg("--organization-name")
            .arg("Test")
            .assert()
            .failure()
            .stderr(predicate_not_a_natural_or_legal_person()?);
    }

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}

#[test]
fn regenerating_ca() -> Result<()> {
    let temp = TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");

    // Generate ca and assert success
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_ca(&ca_prefix)
        .assert()
        .success();

    // Re-generate ca should fail on key
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_ca(&ca_prefix)
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&ca_key)?);

    // Re-generate ca should fail on crt when key is deleted
    std::fs::remove_file(&ca_key)?;
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_ca(&ca_prefix)
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&ca_crt)?);

    // Re-generate ca should succeed with force flag
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_ca(&ca_prefix)
        .arg("--force")
        .assert()
        .success();

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}

#[test]
fn regenerating_cert() -> Result<()> {
    let temp = TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");

    // Generate ca and assert success
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_ca(&ca_prefix)
        .assert()
        .success();

    let (mdl_prefix, mdl_crt, mdl_key) = keypair_paths(&temp, "test-mdl-kp");
    let issuer_auth_json = temp.child("test-issuer-auth.json");

    // Generate issuer JSON input file
    issuer_auth_json.write_str(&serde_json::to_string(&IssuerRegistration::new_mock())?)?;

    // Generate issuer key pair and assert success
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
        .assert()
        .success();

    // Regenerate issuer key pair should fail on key
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&mdl_key)?);

    // Regenerate issuer key pair should fail on crt when key is deleted
    std::fs::remove_file(&mdl_key)?;

    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&mdl_crt)?);

    // Regenerate issuer key pair should succeed with force
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
        .arg("--force")
        .assert()
        .success();

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}

// TODO: PVW-5870 Remove when issuer is just like another cert
fn setup_issuer_files(temp: &TempDir) -> Result<(ChildPath, ChildPath, ChildPath, ChildPath)> {
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(temp, "test-ca");
    let (mdl_prefix, _mdl_crt, _mdl_key) = keypair_paths(temp, "test-mdl-kp");
    let issuer_auth_json = temp.child("test-issuer-auth.json");

    // Generate ca
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_ca(&ca_prefix)
        .arg("--force")
        .assert()
        .success();

    // Generate issuer registration JSON input file
    issuer_auth_json.write_str(&serde_json::to_string(&IssuerRegistration::new_mock())?)?;

    Ok((ca_crt, ca_key, mdl_prefix, issuer_auth_json))
}

// TODO: PVW-5870 Remove when issuer is just like another cert
fn setup_issuer_pubkey_files(temp: &TempDir) -> Result<(ChildPath, ChildPath, ChildPath, ChildPath, ChildPath)> {
    let (ca_crt, ca_key, mdl_prefix, issuer_auth_json) = setup_issuer_files(temp)?;

    let public_key_path = public_key_path(temp, "test-mdl-crt");
    generate_public_key(&public_key_path);

    Ok((public_key_path, ca_crt, ca_key, mdl_prefix, issuer_auth_json))
}

#[test]
// TODO: PVW-5870 Remove when issuer is just like another cert
fn missing_input_files_issuer() -> Result<()> {
    let temp = TempDir::new()?;

    // Setup files without CA key
    let (ca_crt, ca_key, mdl_prefix, issuer_auth_json) = setup_issuer_files(&temp)?;
    std::fs::remove_file(&ca_key)?;

    // Generate issuer should fail when missing CA key file
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
        .assert()
        .failure()
        .stderr(predicate_missing_key_file(&ca_key));

    // Setup files without CA crt
    let (ca_crt, ca_key, mdl_prefix, issuer_auth_json) = setup_issuer_files(&temp)?;
    std::fs::remove_file(&ca_crt)?;

    // Execute command and assert failure and stderr output
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
        .assert()
        .failure()
        .stderr(predicate_missing_crt_file(&ca_crt));

    // Setup files without issuer registration JSON file
    let (ca_crt, ca_key, mdl_prefix, issuer_auth_json) = setup_issuer_files(&temp)?;
    std::fs::remove_file(&issuer_auth_json)?;

    // Generate issuer should fail when missing JSON file
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
        .assert()
        .failure()
        .stderr(predicate_missing_issuer_json_file(&issuer_auth_json));

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}

#[test]
// TODO: PVW-5870 Remove when issuer is just like another cert
fn missing_input_files_issuer_pubkey() -> Result<()> {
    let temp = TempDir::new()?;

    // Setup files without CA key
    let (public_key_file, ca_crt, ca_key, mdl_prefix, issuer_auth_json) = setup_issuer_pubkey_files(&temp)?;
    std::fs::remove_file(&ca_key)?;

    // Generate issuer should fail when missing CA key file
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_issuer_cert(&public_key_file, &ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
        .assert()
        .failure()
        .stderr(predicate_missing_key_file(&ca_key));

    // Setup files without CA crt
    let (public_key_file, ca_crt, ca_key, mdl_prefix, issuer_auth_json) = setup_issuer_pubkey_files(&temp)?;
    std::fs::remove_file(&ca_crt)?;

    // Execute command and assert failure and stderr output
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_issuer_cert(&public_key_file, &ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
        .assert()
        .failure()
        .stderr(predicate_missing_crt_file(&ca_crt));

    // Setup files without issuer registration JSON file
    let (public_key_file, ca_crt, ca_key, mdl_prefix, issuer_auth_json) = setup_issuer_pubkey_files(&temp)?;
    std::fs::remove_file(&issuer_auth_json)?;

    // Generate issuer should fail when missing JSON file
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_issuer_cert(&public_key_file, &ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
        .assert()
        .failure()
        .stderr(predicate_missing_issuer_json_file(&issuer_auth_json));

    // Setup files without public key file
    let (public_key_file, ca_crt, ca_key, mdl_prefix, issuer_auth_json) = setup_issuer_pubkey_files(&temp)?;
    std::fs::remove_file(&public_key_file)?;

    // Generate issuer should fail when missing JSON file
    Command::new(assert_cmd::cargo::cargo_bin!())
        .generate_issuer_cert(&public_key_file, &ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .generate_for_legal_person("Test B.V.", "NTRNL-00000002")
        .assert()
        .failure()
        .stderr(predicate_missing_public_key_file(&public_key_file));

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}
