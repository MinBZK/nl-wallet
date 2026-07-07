use itertools::Itertools;
use utils::vec_at_least::VecNonEmpty;
use webpki::CertRevocationList;
use webpki::OwnedCertRevocationList;
use x509_parser::extensions::DistributionPointName;
use x509_parser::extensions::GeneralName;
use x509_parser::extensions::ParsedExtension;

use crate::x509::BorrowingCertificate;

/// Extract all HTTP(S) CRL distribution point URLs from the certificate's CDP extension.
/// See RFC 5280, section 4.2.1.13.
pub fn extract_crl_distribution_points(cert: &BorrowingCertificate) -> Option<VecNonEmpty<String>> {
    let crl_distribution_points = cert
        .x509_certificate()
        .extensions()
        .iter()
        .filter_map(|ext| {
            if let ParsedExtension::CRLDistributionPoints(cdps) = ext.parsed_extension() {
                Some(cdps)
            } else {
                None
            }
        })
        .flat_map(|cdps| cdps.iter())
        .filter_map(|dp| dp.distribution_point.as_ref())
        .filter_map(|dpn| match dpn {
            DistributionPointName::FullName(names) => Some(names),
            DistributionPointName::NameRelativeToCRLIssuer(..) => {
                // RFC 5280(4.2.1.13): nameRelativeToCRLIssuer is used to form an X.500 distinguished name (LDAP),
                // which we don't support.
                None
            }
        })
        .flat_map(|names| names.iter())
        .filter_map(|name| {
            // RFC 5280(4.2.1.13): If the DistributionPointName contains multiple values, each name
            // describes a different mechanism to obtain the same CRL.  For example,
            // the same CRL could be available for retrieval through both LDAP and
            // HTTP.
            // We only support HTTP via the URI type.
            match name {
                GeneralName::URI(uri) => {
                    // RFC 5280(4.2.1.13): If the DistributionPointName contains a general name of type URI, the
                    // following semantics MUST be assumed: the URI is a pointer to the
                    // current CRL for the associated reasons and will be issued by the
                    // associated cRLIssuer.  When the HTTP or FTP URI scheme is used, the
                    // URI MUST point to a single DER encoded CRL as specified in
                    // [RFC2585].  HTTP server implementations accessed via the URI SHOULD
                    // specify the media type application/pkix-crl in the content-type
                    // header field of the response.
                    Some(uri.to_string())
                }
                _ => None,
            }
        })
        .collect_vec();

    VecNonEmpty::try_from(crl_distribution_points).ok()
}

/// Parse CRL DER bytes into a [`CertRevocationList`] ready for use with
/// [`BorrowingCertificate::verify_with_crls`].
pub fn parse_crl_der(crl_der: &[u8]) -> Result<CertRevocationList<'static>, webpki::Error> {
    let owned = OwnedCertRevocationList::from_der(crl_der)?;
    Ok(CertRevocationList::from(owned))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rcgen::RevokedCertParams;
    use rcgen::RevocationReason;
    use rcgen::SerialNumber;
    use rustls_pki_types::UnixTime;
    use time::OffsetDateTime;
    use url::Url;
    use webpki::RevocationReason as WebpkiRevocationReason;

    use super::*;
    use crate::server_keys::generate::Ca;
    use crate::x509::CertificateConfiguration;
    use crate::x509::DistinguishedName;
    use crate::x509::NO_SAN;

    fn generate_cert_with_cdps(urls: Vec<Url>) -> BorrowingCertificate {
        let ca = Ca::generate_mock();
        let config = CertificateConfiguration {
            crl_distribution_points: urls,
            ..Default::default()
        };
        ca.generate_key_pair(DistinguishedName::create_mock("leaf"), config, NO_SAN)
            .unwrap()
            .into()
    }

    #[test]
    fn no_crl_distribution_points() {
        let cert = generate_cert_with_cdps(vec![]);
        assert!(extract_crl_distribution_points(&cert).is_none());
    }

    #[test]
    fn single_crl_distribution_point() {
        let url: Url = "http://crl.example.com/crl.crl".parse().unwrap();
        let cert = generate_cert_with_cdps(vec![url.clone()]);
        let result = extract_crl_distribution_points(&cert).unwrap();
        assert_eq!(result.as_ref(), &[url.to_string()]);
    }

    #[test]
    fn multiple_crl_distribution_points() {
        let url1: Url = "http://crl.example.com/crl1.crl".parse().unwrap();
        let url2: Url = "http://crl.example.com/crl2.crl".parse().unwrap();
        let cert = generate_cert_with_cdps(vec![url1.clone(), url2.clone()]);
        let result = extract_crl_distribution_points(&cert).unwrap();
        assert_eq!(result.as_ref(), &[url1.to_string(), url2.to_string()]);
    }

    #[test]
    fn parse_empty_crl() {
        let ca = Ca::generate_mock();
        let crl = ca.generate_crl(vec![]).unwrap();
        parse_crl_der(crl.der()).unwrap();
    }

    #[test]
    fn parse_crl_with_revoked_cert() {
        let ca = Ca::generate_mock();

        // Create test CRL
        let serial: &[u8] = &[42];
        let revoked = RevokedCertParams {
            serial_number: SerialNumber::from_slice(serial),
            revocation_time: OffsetDateTime::UNIX_EPOCH,
            reason_code: Some(RevocationReason::KeyCompromise),
            invalidity_date: None,
        };
        let crl = ca.generate_crl(vec![revoked]).unwrap();

        // Parse the CRL
        let parsed = parse_crl_der(crl.der()).unwrap();

        // Find the revoked serial in the CRL
        let revoked_cert = parsed.find_serial(serial).unwrap().unwrap();

        // Verify the revoked certificate data
        assert_eq!(revoked_cert.serial_number, serial);
        assert_eq!(revoked_cert.reason_code, Some(WebpkiRevocationReason::KeyCompromise));
        assert_eq!(revoked_cert.revocation_date, UnixTime::since_unix_epoch(Duration::ZERO));
    }

    #[test]
    fn parse_invalid_crl_der() {
        assert!(parse_crl_der(b"not a crl").is_err());
    }
}
