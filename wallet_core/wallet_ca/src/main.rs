use anyhow::Result;
use clap::{Parser, Subcommand};
use clio::CachedInput;

use nl_wallet_mdoc::utils::x509::{Certificate, CertificateType};
use wallet_ca::{read_certificate, read_reader_registration, read_signing_key, write_key_pair};

/// Generate private keys and certificates
///
/// NOTE: Do NOT use in production environments, as the certificates lifetime is incredibly large, and no revocation is
/// implemented.
#[derive(Parser)]
#[command(author, version, about, long_about)]
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
        /// Overwrite existing files
        #[arg(long, default_value = "false")]
        force: bool,
    },
    /// Generate an Mdl private key and certificate
    MdlCert {
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
        /// Overwrite existing files
        #[arg(long, default_value = "false")]
        force: bool,
    },
    /// Generate a private key and certificate for Relying Party Authentication
    ReaderAuthCert {
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
        /// Overwrite existing files
        #[arg(long, default_value = "false")]
        force: bool,
    },
}

impl Command {
    fn execute(self) -> Result<()> {
        use Command::*;
        match self {
            Ca {
                common_name,
                file_prefix,
                force,
            } => {
                let (certificate, key) = Certificate::new_ca(&common_name)?;
                write_key_pair(key, certificate, &file_prefix, force)?;
                Ok(())
            }
            MdlCert {
                ca_key_file,
                ca_crt_file,
                common_name,
                file_prefix,
                force,
            } => {
                let ca_crt = read_certificate(ca_crt_file)?;
                let ca_key = read_signing_key(ca_key_file)?;
                let (certificate, key) = Certificate::new(&ca_crt, &ca_key, &common_name, CertificateType::Mdl)?;
                write_key_pair(key, certificate, &file_prefix, force)?;
                Ok(())
            }
            ReaderAuthCert {
                ca_key_file,
                ca_crt_file,
                common_name,
                reader_auth_file,
                file_prefix,
                force,
            } => {
                let ca_crt = read_certificate(ca_crt_file)?;
                let ca_key = read_signing_key(ca_key_file)?;
                let reader_registration = read_reader_registration(reader_auth_file)?;
                let (certificate, key) = Certificate::new(
                    &ca_crt,
                    &ca_key,
                    &common_name,
                    CertificateType::ReaderAuth(Box::new(reader_registration)),
                )?;
                write_key_pair(key, certificate, &file_prefix, force)?;
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
