use std::{
    cmp::Ordering,
    ops::{Add, Sub},
    path::Path,
    process::Command,
};

use anyhow::Result;
use assert_cmd::prelude::*;
use assert_fs::{fixture::ChildPath, prelude::*, TempDir};
use p256::{
    ecdsa::SigningKey,
    pkcs8::{
        spki::{DynAssociatedAlgorithmIdentifier, ObjectIdentifier},
        DecodePrivateKey,
    },
};
use predicates::{
    prelude::*,
    str::{RegexPredicate, StartsWithPredicate},
};
use time::{Duration, OffsetDateTime};
use x509_parser::oid_registry::OID_KEY_TYPE_EC_PUBLIC_KEY;

use nl_wallet_mdoc::utils::{issuer_auth::IssuerRegistration, reader_auth::ReaderRegistration};

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

fn predicate_successfully_generated(crt: &Path, key: &Path) -> Result<RegexPredicate> {
    let result = predicate::str::is_match(format!(
        "Certificate stored in '{}'\nKey stored in '{}'",
        crt.display(),
        key.display(),
    ))?;
    Ok(result)
}

fn predicate_file_already_exists(path: &Path) -> Result<RegexPredicate> {
    let result = predicate::str::is_match(format!("Error: Target file '{}' already exists\n", path.display()))?;
    Ok(result)
}

fn predicate_missing_json_file(path: &Path) -> StartsWithPredicate {
    predicate::str::starts_with(format!(
        "error: Invalid value for --reader-auth-file <READER_AUTH_FILE>: Could not open \"{}\": No such file or \
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
    fn generate_mdl_crt(
        &mut self,
        ca_crt: &Path,
        ca_key: &Path,
        issuer_auth_json: &Path,
        file_prefix: &Path,
    ) -> &mut Self;
    fn generate_rp_auth_crt(
        &mut self,
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

    fn generate_mdl_crt(
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
            .arg("test-mdl")
            .arg("--file-prefix")
            .arg(file_prefix)
            .arg("--issuer-auth-file")
            .arg(issuer_auth_json)
    }

    fn generate_rp_auth_crt(
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
            .arg("test-reader-auth")
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
        .stderr(predicate_successfully_generated(&ca_crt, &ca_key)?);

    // Assert generated ca files
    assert_generated_key(&ca_key)?;
    assert_generated_certificate(
        &ca_crt,
        "test-ca",
        OffsetDateTime::now_utc(),
        OffsetDateTime::now_utc() + expected_lifetime,
    )?;

    // Generate mdl certificate
    {
        let (mdl_prefix, mdl_crt, mdl_key) = keypair_paths(&temp, "test-mdl");
        let issuer_auth_json = temp.child("test-issuer-auth.json");

        // Generate reader-auth JSON input file
        issuer_auth_json.write_str(&serde_json::to_string(&IssuerRegistration::new_mock())?)?;

        // Execute command and assert success and stderr output
        Command::cargo_bin("wallet_ca")?
            .generate_mdl_crt(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
            .assert()
            .success()
            .stderr(predicate_successfully_generated(&mdl_crt, &mdl_key)?);

        // Assert generated mdl files
        assert_generated_key(&mdl_key)?;
        assert_generated_certificate(
            &mdl_crt,
            "test-mdl",
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + expected_lifetime,
        )?;
    }

    // Generate reader-auth certificate
    {
        let (rp_auth_prefix, rp_auth_crt, rp_auth_key) = keypair_paths(&temp, "test-reader-auth");
        let rp_auth_json = temp.child("test-reader-auth.json");

        // Generate reader-auth JSON input file
        rp_auth_json.write_str(&serde_json::to_string(&ReaderRegistration::new_mock())?)?;

        // Execute command and assert success and stderr output
        Command::cargo_bin("wallet_ca")?
            .generate_rp_auth_crt(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
            .assert()
            .success()
            .stderr(predicate_successfully_generated(&rp_auth_crt, &rp_auth_key)?);

        // Assert generated rp_auth files
        assert_generated_key(&rp_auth_key)?;
        assert_generated_certificate(
            &rp_auth_crt,
            "test-reader-auth",
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
        .stderr(predicate_successfully_generated(&ca_crt, &ca_key)?);

    // Assert generated ca files
    assert_generated_key(&ca_key)?;
    assert_generated_certificate(
        &ca_crt,
        "test-ca",
        OffsetDateTime::now_utc(),
        OffsetDateTime::now_utc() + Duration::days(586),
    )?;

    // Generate mdl certificate
    {
        let (mdl_prefix, mdl_crt, mdl_key) = keypair_paths(&temp, "test-mdl");
        let issuer_auth_json = temp.child("test-issuer-auth.json");

        // Generate reader-auth JSON input file
        issuer_auth_json.write_str(&serde_json::to_string(&IssuerRegistration::new_mock())?)?;

        // Execute command and assert success and stderr output
        Command::cargo_bin("wallet_ca")?
            .generate_mdl_crt(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
            .arg("--days=678")
            .assert()
            .success()
            .stderr(predicate_successfully_generated(&mdl_crt, &mdl_key)?);

        // Assert generated mdl files
        assert_generated_key(&mdl_key)?;
        assert_generated_certificate(
            &mdl_crt,
            "test-mdl",
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc() + Duration::days(678),
        )?;
    }

    // Generate reader-auth certificate
    {
        let (rp_auth_prefix, rp_auth_crt, rp_auth_key) = keypair_paths(&temp, "test-reader-auth");
        let rp_auth_json = temp.child("test-reader-auth.json");

        // Generate reader-auth JSON input file
        rp_auth_json.write_str(&serde_json::to_string(&ReaderRegistration::new_mock())?)?;

        // Execute command and assert success and stderr output
        Command::cargo_bin("wallet_ca")?
            .generate_rp_auth_crt(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
            .arg("--days")
            .arg("7")
            .assert()
            .success()
            .stderr(predicate_successfully_generated(&rp_auth_crt, &rp_auth_key)?);

        // Assert generated rp_auth files
        assert_generated_key(&rp_auth_key)?;
        assert_generated_certificate(
            &rp_auth_crt,
            "test-reader-auth",
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

    // Re-generate ca should fail on crt
    Command::cargo_bin("wallet_ca")?
        .generate_ca(&ca_prefix)
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&ca_crt)?);

    // Re-generate ca should fail on key when crt is deleted
    std::fs::remove_file(&ca_crt)?;
    Command::cargo_bin("wallet_ca")?
        .generate_ca(&ca_prefix)
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&ca_key)?);

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

    let (mdl_prefix, mdl_crt, mdl_key) = keypair_paths(&temp, "test-mdl");
    let issuer_auth_json = temp.child("test-issuer-auth.json");

    // Generate reader-auth JSON input file
    issuer_auth_json.write_str(&serde_json::to_string(&IssuerRegistration::new_mock())?)?;

    // Generate mdl certificate and assert success
    Command::cargo_bin("wallet_ca")?
        .generate_mdl_crt(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .assert()
        .success();

    // Regenerate mdl certificate should fail on crt
    Command::cargo_bin("wallet_ca")?
        .generate_mdl_crt(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&mdl_crt)?);

    // Regenerate mdl certificate should fail on key when crt is deleted
    std::fs::remove_file(&mdl_crt)?;

    Command::cargo_bin("wallet_ca")?
        .generate_mdl_crt(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&mdl_key)?);

    // Regenerate mdl certificate should succeed with force
    Command::cargo_bin("wallet_ca")?
        .generate_mdl_crt(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
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

    let (rp_auth_prefix, rp_auth_crt, rp_auth_key) = keypair_paths(&temp, "test-reader-auth");
    let rp_auth_json = temp.child("test-reader-auth.json");

    // Generate reader-auth JSON input file
    rp_auth_json.write_str(&serde_json::to_string(&ReaderRegistration::new_mock())?)?;

    // Generate rp_auth certificate and assert success
    Command::cargo_bin("wallet_ca")?
        .generate_rp_auth_crt(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
        .assert()
        .success();

    // Regenerate rp_auth certificate should fail on crt
    Command::cargo_bin("wallet_ca")?
        .generate_rp_auth_crt(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&rp_auth_crt)?);

    // Regenerate rp_auth certificate should fail on key when crt is deleted
    std::fs::remove_file(&rp_auth_crt)?;

    Command::cargo_bin("wallet_ca")?
        .generate_rp_auth_crt(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
        .assert()
        .failure()
        .stderr(predicate_file_already_exists(&rp_auth_key)?);

    // Regenerate rp_auth certificate should succeed with force
    Command::cargo_bin("wallet_ca")?
        .generate_rp_auth_crt(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
        .arg("--force")
        .assert()
        .success();

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}

#[test]
fn missing_input_files() -> Result<()> {
    let temp = assert_fs::TempDir::new()?;
    let (ca_prefix, ca_crt, ca_key) = keypair_paths(&temp, "test-ca");
    let (rp_auth_prefix, _rp_auth_crt, _rp_auth_key) = keypair_paths(&temp, "test-reader-auth");
    let rp_auth_json = temp.child("test-reader-auth.json");
    let issuer_auth_json = temp.child("test-issuer-auth.json");

    // Generate ca
    Command::cargo_bin("wallet_ca")?
        .generate_ca(&ca_prefix)
        .assert()
        .success();

    // Generate reader_auth should fail when missing JSON file
    Command::cargo_bin("wallet_ca")?
        .generate_rp_auth_crt(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
        .assert()
        .failure()
        .stderr(predicate_missing_json_file(&rp_auth_json));

    // Generate reader-auth JSON input file
    rp_auth_json.write_str(&serde_json::to_string(&ReaderRegistration::new_mock())?)?;

    // Generate certificates with missing crt should fail on crt
    std::fs::remove_file(&ca_crt)?;

    let (mdl_prefix, _mdl_crt, _mdl_key) = keypair_paths(&temp, "test-mdl");

    // Generate mdl should fail when missing JSON file
    Command::cargo_bin("wallet_ca")?
        .generate_mdl_crt(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .assert()
        .failure()
        .stderr(predicate_missing_crt_file(&ca_crt));

    // Generate reader-auth JSON input file
    issuer_auth_json.write_str(&serde_json::to_string(&IssuerRegistration::new_mock())?)?;

    // Execute command and assert failure and stderr output
    Command::cargo_bin("wallet_ca")?
        .generate_mdl_crt(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .assert()
        .failure()
        .stderr(predicate_missing_crt_file(&ca_crt));

    Command::cargo_bin("wallet_ca")?
        .generate_rp_auth_crt(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
        .assert()
        .failure()
        .stderr(predicate_missing_crt_file(&ca_crt));

    // Generate certificates with missing key and crt should fail on key
    std::fs::remove_file(&ca_key)?;

    Command::cargo_bin("wallet_ca")?
        .generate_mdl_crt(&ca_crt, &ca_key, &issuer_auth_json, &mdl_prefix)
        .assert()
        .failure()
        .stderr(predicate_missing_key_file(&ca_key));

    Command::cargo_bin("wallet_ca")?
        .generate_rp_auth_crt(&ca_crt, &ca_key, &rp_auth_json, &rp_auth_prefix)
        .assert()
        .failure()
        .stderr(predicate_missing_key_file(&ca_key));

    // Explicitly close the temp folder, for better error reporting
    temp.close()?;

    Ok(())
}
