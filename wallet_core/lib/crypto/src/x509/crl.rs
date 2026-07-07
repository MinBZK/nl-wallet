use itertools::Itertools;
use utils::vec_at_least::VecNonEmpty;
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

#[cfg(test)]
mod tests {
    use url::Url;

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
}
