use std::cmp::Ordering;
use std::fs;
use std::ops::Add;
use std::ops::Sub;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use assert_cmd::prelude::*;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::spki::DynAssociatedAlgorithmIdentifier;
use p256::pkcs8::spki::ObjectIdentifier;
use p256::pkcs8::DecodePrivateKey;
use p256::pkcs8::EncodePublicKey;
use pem::EncodeConfig;
use pem::LineEnding;
use pem::Pem;
use predicates::prelude::*;
use predicates::str::RegexPredicate;
use predicates::str::StartsWithPredicate;
use time::Duration;
use time::OffsetDateTime;
use x509_parser::oid_registry::OID_KEY_TYPE_EC_PUBLIC_KEY;

use nl_wallet_mdoc::utils::issuer_auth::IssuerRegistration;
use nl_wallet_mdoc::utils::reader_auth::ReaderRegistration;

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

fn predicate_file_already_exists(path: &Path) -> Result<RegexPredicate> {
    let result = predicate::str::is_match(format!("Error: Target file '{}' already exists\n", path.display()))?;
    Ok(result)
}

fn predicate_missing_reader_json_file(path: &Path) -> StartsWithPredicate {
    predicate::str::starts_with(format!(
        "error: Invalid value for --reader-auth-file <READER_AUTH_FILE>: Could not open \"{}\": No such file or \
         directory",
        path.display()
    ))
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

// fn predicate_missing_public_key_file(path: &Path) -> StartsWithPredicate {
//     predicate::str::starts_with(format!(
//         r#"error: Invalid value for --public-key-file <PUBLIC_KEY_FILE>: Could not open "{}": No such file or directory"#,
//         path.display()
//     ))
// }

fn assert_generated_key(key_file: &ChildPath) -> Result<()> {
    // Read key and verify algorithm
    SigningKey::read_pkcs8_pem_file(key_file)?
        .algorithm_identifier()?
        .assert_algorithm_oid(ObjectIdentifier::new_unwrap(&OID_KEY_TYPE_EC_PUBLIC_KEY.to_id_string()))?;

    Ok(())
}

fn assert_generated_certificate(
    crt_file: &ChildPath,
    expected_cn: &str,
    start: OffsetDateTime,
    end: OffsetDateTime,
) -> Result<()> {
    // Read certificate and verify PEM label
    let crt_pem_bytes = std::fs::read(crt_file)?;
    let (_, crt_pem) = x509_parser::pem::parse_x509_pem(&crt_pem_bytes)?;
    assert_eq!(crt_pem.label, "CERTIFICATE");
    let crt = crt_pem.parse_x509()?;

    // verify CN
    itertools::assert_equal(
        crt.subject().iter_common_name().map(|a| a.as_str().unwrap()),
        vec![expected_cn],
    );

    // verify validity times with minute accuracy
    let not_before = crt.validity().not_before.to_datetime();
    assert_eq!(not_before.cmp_range(&start, Duration::minutes(1)), Ordering::Equal);
    let not_after = crt.validity().not_after.to_datetime();
    assert_eq!(not_after.cmp_range(&end, Duration::minutes(1)), Ordering::Equal);

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
    fn generate_reader_kp(
        &mut self,
        ca_crt: &Path,
        ca_key: &Path,
        rp_auth_json: &Path,
        file_prefix: &Path,
    ) -> &mut Self;
    fn generate_issuer_cert(
        &mut self,
        pk: &Path,
        ca_crt: &Path,
        ca_key: &Path,
        issuer_auth_json: &Path,
        file_prefix: &Path,
    ) -> &mut Self;
    fn generate_reader_cert(
        &mut self,
        pk: &Path,
        ca_crt: &Path,
        ca_key: &Path,
        rp_auth_json: &Path,
        file_prefix: &Path,
    ) -> &mut Self;
}

impl CommandExtension for Command {
    fn generate_ca(&mut self, file_prefix: &Path) -> &mut Self {
        self.arg("ca")
            .arg("--common-name")
            .arg("test-ca")
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
        self.arg("issuer")
            .arg("--ca-key-file")
            .arg(ca_key)
            .arg("--ca-crt-file")
            .arg(ca_crt)
            .arg("--common-name")
            .arg("test-mdl-kp")
            .arg("--file-prefix")
            .arg(file_prefix)
            .arg("--issuer-auth-file")
            .arg(issuer_auth_json)
    }

    fn generate_reader_kp(
        &mut self,
        ca_crt: &Path,
        ca_key: &Path,
        rp_auth_json: &Path,
        file_prefix: &Path,
    ) -> &mut Self {
        self.arg("reader")
            .arg("--ca-key-file")
            .arg(ca_key)
            .arg("--ca-crt-file")
            .arg(ca_crt)
            .arg("--common-name")
            .arg("test-reader-auth-kp")
            .arg("--file-prefix")
            .arg(file_prefix)
            .arg("--reader-auth-file")
            .arg(rp_auth_json)
    }

    fn generate_issuer_cert(
        &mut self,
        pk: &Path,
        ca_crt: &Path,
        ca_key: &Path,
        issuer_auth_json: &Path,
        file_prefix: &Path,
    ) -> &mut Self {
        self.arg("issuer-cert")
            .arg("--public-key-file")
            .arg(pk)
            .arg("--ca-key-file")
            .arg(ca_key)
            .arg("--ca-crt-file")
            .arg(ca_crt)
            .arg("--common-name")
            .arg("test-mdl-crt")
            .arg("--file-prefix")
            .arg(file_prefix)
            .arg("--issuer-auth-file")
            .arg(issuer_auth_json)
    }

    fn generate_reader_cert(
        &mut self,
        pk: &Path,
        ca_crt: &Path,
        ca_key: &Path,
        rp_auth_json: &Path,
        file_prefix: &Path,
    ) -> &mut Self {
        self.arg("reader-cert")
            .arg("--public-key-file")
            .arg(pk)
            .arg("--ca-key-file")
            .arg(ca_key)
            .arg("--ca-crt-file")
            .arg(ca_crt)
            .arg("--common-name")
            .arg("test-reader-auth-crt")
            .arg("--file-prefix")
            .arg(file_prefix)
            .arg("--reader-auth-file")
            .arg(rp_auth_json)
    }
}

fn keypair_paths(temp: &TempDir, prefix: &str) -> (ChildPath, ChildPath, ChildPath) {
    (
        temp.child(prefix),
        temp.child(format!("{}.crt.pem", prefix)),
        temp.child(format!("{}.key.pem", prefix)),
    )
}

fn certificate_path(temp: &TempDir, prefix: &str) -> ChildPath {
    temp.child(format!("{}.pk.pem", prefix))
}

fn generate_public_key(path: &ChildPath) {
    let signing_key = SigningKey::random(&mut OsRng);
    let public_key = signing_key.verifying_key();
    let der = public_key.to_public_key_der().unwrap();
    let pem = Pem::new("PUBLIC KEY", der.to_vec());
    fs::write(
        path,
        pem::encode_config(&pem, EncodeConfig::new().set_line_ending(LineEnding::LF)),
    )
    .unwrap();
}

#[test]
fn happy_flow_with_default_lifetime() -> Result<()> {
    let temp = assert_fs::TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");

    let expected_lifetime = Duration::days(365);

    // Generate ca and assert success and stderr output
    Command::cargo_bin("wallet_ca")?
        .generate_ca(&ca_prefix)
        .assert()
        .success()
        .stderr(predicate_successfully_generated_key_pair(&ca_crt, &ca_key)?);

    // Assert generated ca files
    assert_generated_key(&ca_key)?;
    assert_generated_certificate(
        &ca_crt,
        "test-ca",
        OffsetDateTime::now_utc(),
        OffsetDateTime::now_utc() + expected_lifetime,
    )?;

    // Generate issuer key pair
    {
        let (mdl_prefix, mdl_crt, mdl_key) = keypair_paths(&temp, "test-mdl-kp");
        let issuer_auth_json = temp.child("test-issuer-auth.json");

        // Generate issuer registration JSON input file
        issuer_auth_json.write_str(&serde_json::to_string(&IssuerRegistration::new_mock())?)?;

        // Execute command and assert success and stderr output
        Command::cargo_bin("wallet_ca")?
            .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
            .assert()
            .success()
            .stderr(predicate_successfully_generated_key_pair(&mdl_crt, &mdl_key)?);

        // Assert generated issuer files
        assert_generated_key(&mdl_key)?;
        assert_generated_certificate(
            &mdl_crt,
            "test-mdl-kp",
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + expected_lifetime,
        )?;
    }

    // Generate issuer certificate
    {
        let (mdl_prefix, mdl_crt, _) = keypair_paths(&temp, "test-mdl-crt");
        let issuer_auth_json = temp.child("test-issuer-auth.json");

        // Generate issuer registration JSON input file
        issuer_auth_json.write_str(&serde_json::to_string(&IssuerRegistration::new_mock())?)?;

        let public_key_path = certificate_path(&temp, "test-mdl-crt");
        generate_public_key(&public_key_path);
        // Execute command and assert success and stderr output
        Command::cargo_bin("wallet_ca")?
            .generate_issuer_cert(&public_key_path, &ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
            .assert()
            .success()
            .stderr(predicate_successfully_generated_certificate(&mdl_crt)?);

        // Assert generated issuer certificate
        assert_generated_certificate(
            &mdl_crt,
            "test-mdl-crt",
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + expected_lifetime,
        )?;
    }

    // Generate reader key pair
    {
        let (rp_auth_prefix, rp_auth_crt, rp_auth_key) = keypair_paths(&temp, "test-reader-auth-kp");
        let rp_auth_json = temp.child("test-reader-auth.json");

        // Generate reader registration JSON input file
        rp_auth_json.write_str(&serde_json::to_string(&ReaderRegistration::new_mock())?)?;

        // Execute command and assert success and stderr output
        Command::cargo_bin("wallet_ca")?
            .generate_reader_kp(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
            .assert()
            .success()
            .stderr(predicate_successfully_generated_key_pair(&rp_auth_crt, &rp_auth_key)?);

        // Assert generated reader files
        assert_generated_key(&rp_auth_key)?;
        assert_generated_certificate(
            &rp_auth_crt,
            "test-reader-auth-kp",
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + expected_lifetime,
        )?;
    }

    // Generate reader certificate
    {
        let (rp_auth_prefix, rp_auth_crt, _) = keypair_paths(&temp, "test-reader-auth-crt");
        let rp_auth_json = temp.child("test-reader-auth.json");

        // Generate reader registration JSON input file
        rp_auth_json.write_str(&serde_json::to_string(&ReaderRegistration::new_mock())?)?;

        let public_key_path = certificate_path(&temp, "test-reader-auth-crt");
        generate_public_key(&public_key_path);

        // Execute command and assert success and stderr output
        Command::cargo_bin("wallet_ca")?
            .generate_reader_cert(&public_key_path, &ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
            .assert()
            .success()
            .stderr(predicate_successfully_generated_certificate(&rp_auth_crt)?);

        // Assert generated reader certificate
        assert_generated_certificate(
            &rp_auth_crt,
            "test-reader-auth-crt",
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + expected_lifetime,
        )?;
    }

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}

#[test]
fn happy_flow_with_custom_lifetime() -> Result<()> {
    let temp = assert_fs::TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");

    // Generate ca and assert success and stderr output
    Command::cargo_bin("wallet_ca")?
        .generate_ca(&ca_prefix)
        .arg("--days")
        .arg("586")
        .assert()
        .success()
        .stderr(predicate_successfully_generated_key_pair(&ca_crt, &ca_key)?);

    // Assert generated ca files
    assert_generated_key(&ca_key)?;
    assert_generated_certificate(
        &ca_crt,
        "test-ca",
        OffsetDateTime::now_utc(),
        OffsetDateTime::now_utc() + Duration::days(586),
    )?;

    // Generate issuer key pair
    {
        let (mdl_prefix, mdl_crt, mdl_key) = keypair_paths(&temp, "test-mdl-kp");
        let issuer_auth_json = temp.child("test-issuer-auth.json");

        // Generate issuer registration JSON input file
        issuer_auth_json.write_str(&serde_json::to_string(&IssuerRegistration::new_mock())?)?;

        // Execute command and assert success and stderr output
        Command::cargo_bin("wallet_ca")?
            .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
            .arg("--days=678")
            .assert()
            .success()
            .stderr(predicate_successfully_generated_key_pair(&mdl_crt, &mdl_key)?);

        // Assert generated issuer files
        assert_generated_key(&mdl_key)?;
        assert_generated_certificate(
            &mdl_crt,
            "test-mdl-kp",
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + Duration::days(678),
        )?;
    }

    // Generate issuer certificate
    {
        let (mdl_prefix, mdl_crt, _) = keypair_paths(&temp, "test-mdl-crt");
        let issuer_auth_json = temp.child("test-issuer-auth.json");

        // Generate issuer registration JSON input file
        issuer_auth_json.write_str(&serde_json::to_string(&IssuerRegistration::new_mock())?)?;

        let public_key_path = certificate_path(&temp, "test-mdl-crt");
        generate_public_key(&public_key_path);
        // Execute command and assert success and stderr output
        Command::cargo_bin("wallet_ca")?
            .generate_issuer_cert(&public_key_path, &ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
            .arg("--days=678")
            .assert()
            .success()
            .stderr(predicate_successfully_generated_certificate(&mdl_crt)?);

        // Assert generated issuer certificate
        assert_generated_certificate(
            &mdl_crt,
            "test-mdl-crt",
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + Duration::days(678),
        )?;
    }

    // Generate reader key pair
    {
        let (rp_auth_prefix, rp_auth_crt, rp_auth_key) = keypair_paths(&temp, "test-reader-auth-kp");
        let rp_auth_json = temp.child("test-reader-auth.json");

        // Generate reader JSON input file
        rp_auth_json.write_str(&serde_json::to_string(&ReaderRegistration::new_mock())?)?;

        // Execute command and assert success and stderr output
        Command::cargo_bin("wallet_ca")?
            .generate_reader_kp(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
            .arg("--days")
            .arg("7")
            .assert()
            .success()
            .stderr(predicate_successfully_generated_key_pair(&rp_auth_crt, &rp_auth_key)?);

        // Assert generated reader files
        assert_generated_key(&rp_auth_key)?;
        assert_generated_certificate(
            &rp_auth_crt,
            "test-reader-auth-kp",
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + Duration::days(7),
        )?;
    }

    // Generate reader certificate
    {
        let (rp_auth_prefix, rp_auth_crt, _) = keypair_paths(&temp, "test-reader-auth-crt");
        let rp_auth_json = temp.child("test-reader-auth.json");

        // Generate reader registration JSON input file
        rp_auth_json.write_str(&serde_json::to_string(&ReaderRegistration::new_mock())?)?;

        let public_key_path = certificate_path(&temp, "test-reader-auth-crt");
        generate_public_key(&public_key_path);

        // Execute command and assert success and stderr output
        Command::cargo_bin("wallet_ca")?
            .generate_reader_cert(&public_key_path, &ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
            .arg("--days")
            .arg("7")
            .assert()
            .success()
            .stderr(predicate_successfully_generated_certificate(&rp_auth_crt)?);

        // Assert generated reader certificate
        assert_generated_certificate(
            &rp_auth_crt,
            "test-reader-auth-crt",
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + Duration::days(7),
        )?;
    }

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}

#[test]
fn regenerating_ca() -> Result<()> {
    let temp = assert_fs::TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");

    // Generate ca and assert success
    Command::cargo_bin("wallet_ca")?
        .generate_ca(&ca_prefix)
        .assert()
        .success();

    // Re-generate ca should fail on key
    Command::cargo_bin("wallet_ca")?
        .generate_ca(&ca_prefix)
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&ca_key)?);

    // Re-generate ca should fail on crt when key is deleted
    std::fs::remove_file(&ca_key)?;
    Command::cargo_bin("wallet_ca")?
        .generate_ca(&ca_prefix)
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&ca_crt)?);

    // Re-generate ca should succeed with force flag
    Command::cargo_bin("wallet_ca")?
        .generate_ca(&ca_prefix)
        .arg("--force")
        .assert()
        .success();

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}

#[test]
fn regenerating_mdl() -> Result<()> {
    let temp = assert_fs::TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");

    // Generate ca and assert success
    Command::cargo_bin("wallet_ca")?
        .generate_ca(&ca_prefix)
        .assert()
        .success();

    let (mdl_prefix, mdl_crt, mdl_key) = keypair_paths(&temp, "test-mdl-kp");
    let issuer_auth_json = temp.child("test-issuer-auth.json");

    // Generate reader JSON input file
    issuer_auth_json.write_str(&serde_json::to_string(&IssuerRegistration::new_mock())?)?;

    // Generate issuer key pair and assert success
    Command::cargo_bin("wallet_ca")?
        .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .assert()
        .success();

    // Regenerate issuer key pair should fail on key
    Command::cargo_bin("wallet_ca")?
        .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&mdl_key)?);

    // Regenerate issuer key pair should fail on crt when key is deleted
    std::fs::remove_file(&mdl_key)?;

    Command::cargo_bin("wallet_ca")?
        .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&mdl_crt)?);

    // Regenerate issuer key pair should succeed with force
    Command::cargo_bin("wallet_ca")?
        .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .arg("--force")
        .assert()
        .success();

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}

#[test]
fn regenerating_rp_auth() -> Result<()> {
    let temp = assert_fs::TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");

    // Generate ca and assert success
    Command::cargo_bin("wallet_ca")?
        .generate_ca(&ca_prefix)
        .assert()
        .success();

    let (rp_auth_prefix, rp_auth_crt, rp_auth_key) = keypair_paths(&temp, "test-reader-auth-kp");
    let rp_auth_json = temp.child("test-reader-auth.json");

    // Generate reader JSON input file
    rp_auth_json.write_str(&serde_json::to_string(&ReaderRegistration::new_mock())?)?;

    // Generate reader key pair and assert success
    Command::cargo_bin("wallet_ca")?
        .generate_reader_kp(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
        .assert()
        .success();

    // Regenerate reader key pair should fail on key
    Command::cargo_bin("wallet_ca")?
        .generate_reader_kp(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&rp_auth_key)?);

    // Regenerate reader key pair should fail on crt when key is deleted
    std::fs::remove_file(&rp_auth_key)?;

    Command::cargo_bin("wallet_ca")?
        .generate_reader_kp(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&rp_auth_crt)?);

    // Regenerate reader key pair should succeed with force
    Command::cargo_bin("wallet_ca")?
        .generate_reader_kp(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
        .arg("--force")
        .assert()
        .success();

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}

fn setup_issuer_files(temp: &TempDir) -> Result<(ChildPath, ChildPath, ChildPath, ChildPath)> {
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(temp, "test-ca");
    let (mdl_prefix, _mdl_crt, _mdl_key) = keypair_paths(temp, "test-mdl-kp");
    let issuer_auth_json = temp.child("test-issuer-auth.json");

    // Generate ca
    Command::cargo_bin("wallet_ca")?
        .generate_ca(&ca_prefix)
        .arg("--force")
        .assert()
        .success();

    // Generate issuer registration JSON input file
    issuer_auth_json.write_str(&serde_json::to_string(&IssuerRegistration::new_mock())?)?;

    Ok((ca_crt, ca_key, mdl_prefix, issuer_auth_json))
}

#[test]
fn missing_input_files_issuer() -> Result<()> {
    let temp = assert_fs::TempDir::new()?;

    // Setup files without CA key
    let (ca_crt, ca_key, mdl_prefix, issuer_auth_json) = setup_issuer_files(&temp)?;
    std::fs::remove_file(&ca_key)?;

    // Generate issuer should fail when missing CA key file
    Command::cargo_bin("wallet_ca")?
        .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .assert()
        .failure()
        .stderr(predicate_missing_key_file(&ca_key));

    // Setup files without CA crt
    let (ca_crt, ca_key, mdl_prefix, issuer_auth_json) = setup_issuer_files(&temp)?;
    std::fs::remove_file(&ca_crt)?;

    // Execute command and assert failure and stderr output
    Command::cargo_bin("wallet_ca")?
        .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .assert()
        .failure()
        .stderr(predicate_missing_crt_file(&ca_crt));

    // Setup files without issuer registration JSON file
    let (ca_crt, ca_key, mdl_prefix, issuer_auth_json) = setup_issuer_files(&temp)?;
    std::fs::remove_file(&issuer_auth_json)?;

    // Generate issuer should fail when missing JSON file
    Command::cargo_bin("wallet_ca")?
        .generate_issuer_kp(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .assert()
        .failure()
        .stderr(predicate_missing_issuer_json_file(&issuer_auth_json));

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}

fn setup_reader_files(temp: &TempDir) -> Result<(ChildPath, ChildPath, ChildPath, ChildPath)> {
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(temp, "test-ca");
    let (rp_auth_prefix, _rp_auth_crt, _rp_auth_key) = keypair_paths(temp, "test-reader-auth-kp");
    let rp_auth_json = temp.child("test-reader-auth.json");

    // Generate ca
    Command::cargo_bin("wallet_ca")?
        .generate_ca(&ca_prefix)
        .arg("--force")
        .assert()
        .success();

    // Generate reader registration JSON input file
    rp_auth_json.write_str(&serde_json::to_string(&ReaderRegistration::new_mock())?)?;

    Ok((ca_crt, ca_key, rp_auth_prefix, rp_auth_json))
}

#[test]
fn missing_input_files_reader() -> Result<()> {
    let temp = assert_fs::TempDir::new()?;

    // Setup files without CA key
    let (ca_crt, ca_key, rp_auth_prefix, rp_auth_json) = setup_reader_files(&temp)?;
    std::fs::remove_file(&ca_key)?;

    // Generate reader should fail on missing CA key
    Command::cargo_bin("wallet_ca")?
        .generate_reader_kp(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
        .assert()
        .failure()
        .stderr(predicate_missing_key_file(&ca_key));

    // Setup files without CA crt
    let (ca_crt, ca_key, rp_auth_prefix, rp_auth_json) = setup_reader_files(&temp)?;
    std::fs::remove_file(&ca_crt)?;

    Command::cargo_bin("wallet_ca")?
        .generate_reader_kp(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
        .assert()
        .failure()
        .stderr(predicate_missing_crt_file(&ca_crt));

    // Setup files without reader registration JSON file
    let (ca_crt, ca_key, rp_auth_prefix, rp_auth_json) = setup_reader_files(&temp)?;
    std::fs::remove_file(&rp_auth_json)?;

    // Generate reader_auth should fail when missing JSON file
    Command::cargo_bin("wallet_ca")?
        .generate_reader_kp(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
        .assert()
        .failure()
        .stderr(predicate_missing_reader_json_file(&rp_auth_json));

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}
