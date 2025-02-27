use anyhow::Result;
use chrono::Duration;
use chrono::Utc;
use clap::Parser;
use clap::Subcommand;
use clio::CachedInput;

use nl_wallet_mdoc::server_keys::generate;
use nl_wallet_mdoc::utils::issuer_auth::IssuerRegistration;
use nl_wallet_mdoc::utils::reader_auth::ReaderRegistration;
use nl_wallet_mdoc::utils::x509::CertificateConfiguration;
use wallet_ca::read_public_key;
use wallet_ca::read_self_signed_ca;
use wallet_ca::write_certificate;
use wallet_ca::write_key_pair;
use wallet_common::built_info::version_string;

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
}

impl Command {
    fn get_certificate_configuration(days: u32) -> CertificateConfiguration {
        let not_before = Utc::now();
        let not_after = not_before
            .checked_add_signed(Duration::days(days as i64))
            .expect("`valid_for` does not result in a valid time stamp, try decreasing the value");
        if not_after <= not_before {
            panic!("`valid_for` must be a positive duration");
        }
        CertificateConfiguration {
            not_before: Some(not_before),
            not_after: Some(not_after),
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
                write_key_pair(ca.as_certificate_der(), &signing_key, &file_prefix, force)?;
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
                    &issuer_registration.into(),
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
                    &reader_registration.into(),
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
                    &issuer_registration.into(),
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
                    &reader_registration.into(),
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
