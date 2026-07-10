use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use itertools::Itertools;
use moka::Expiry;
use moka::future::Cache;
use reqwest::Client;
use url::Url;
use utils::vec_at_least::VecNonEmpty;
use webpki::CertRevocationList;
use webpki::OwnedCertRevocationList;
use x509_parser::extensions::DistributionPointName;
use x509_parser::extensions::GeneralName;
use x509_parser::extensions::ParsedExtension;
use x509_parser::parse_x509_crl;
use x509_parser::revocation_list::CertificateRevocationList;

use crate::x509::BorrowingCertificate;

#[derive(Debug, thiserror::Error)]
pub enum CrlProviderError {
    #[error("HTTP error fetching CRL: {0}")]
    Http(#[source] reqwest::Error),
    #[error("CRL parsing error: {0}")]
    Parsing(#[source] webpki::Error),
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[source] url::ParseError),
}

struct CrlExpiry;

impl Expiry<Url, Arc<Vec<u8>>> for CrlExpiry {
    fn expire_after_create(&self, _key: &Url, value: &Arc<Vec<u8>>, _created_at: Instant) -> Option<Duration> {
        let (_, crl) = parse_x509_crl(value).ok()?;
        ttl_from_next_update(&crl)
    }
}

/// Downloads and caches RFC 5280 CRLs keyed by URL.
///
/// Used to check revocation status of signing certificates during SD-JWT and MsoMdoc message
/// verification. The cache TTL for each entry is derived from the CRL's `nextUpdate` field so
/// entries are refreshed automatically when the CRL expires.
pub struct CrlProvider {
    client: Client,
    cache: Cache<Url, Arc<Vec<u8>>>,
}

impl CrlProvider {
    pub fn new(client: Client, max_capacity: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .expire_after(CrlExpiry)
            .build();
        Self { client, cache }
    }

    #[cfg(any(test, feature = "mock"))]
    pub fn new_without_caching(client: Client) -> Self {
        Self {
            client,
            cache: Cache::builder().max_capacity(0).build(),
        }
    }

    async fn fetch_bytes(&self, url: Url) -> Result<Arc<Vec<u8>>, CrlProviderError> {
        if let Some(cached) = self.cache.get(&url).await {
            return Ok(cached);
        }
        let bytes: Arc<Vec<u8>> = Arc::new(
            self.client
                .get(url.clone())
                .send()
                .await
                .map_err(CrlProviderError::Http)?
                .error_for_status()
                .map_err(CrlProviderError::Http)?
                .bytes()
                .await
                .map_err(CrlProviderError::Http)?
                .to_vec(),
        );
        // TODO verify CRL bytes before storing in the cache (signature, validity, ...?)
        self.cache.insert(url, bytes.clone()).await;
        Ok(bytes)
    }

    /// Fetch all CRLs referenced in the certificate's CDP extension
    pub async fn crls_for_cert(
        &self,
        cert: &BorrowingCertificate,
    ) -> Result<Vec<CertRevocationList<'static>>, CrlProviderError> {
        let urls = extract_crl_distribution_points(cert);
        let mut crls = Vec::with_capacity(urls.as_ref().map(|urls| urls.len().get()).unwrap_or(0));
        for url in urls.iter().flatten() {
            let bytes = self
                .fetch_bytes(url.parse().map_err(CrlProviderError::InvalidUrl)?)
                .await?; // TODO error handling
            crls.push(parse_crl_der(&bytes).map_err(CrlProviderError::Parsing)?);
        }
        Ok(crls)
    }
}

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

/// Return remaining time until the CRL's `nextUpdate` field expires.
/// Returns `None` if the CRL has no `nextUpdate` or is already past expiry.
/// Used by callers to derive cache TTL.
pub fn ttl_from_next_update(crl: &CertificateRevocationList) -> Option<Duration> {
    let next_update_secs = crl.next_update()?.to_datetime().unix_timestamp();
    let now_secs = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs() as i64;
    let remaining = (next_update_secs - now_secs).max(0) as u64;
    Some(Duration::from_secs(remaining))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use der::Encode;
    use der::asn1::BitStringRef;
    use der::asn1::ObjectIdentifier;
    use der::asn1::SequenceOf;
    use der::asn1::UtcTime;
    use http_utils::httpmock::httpmock_reqwest_client_builder;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use rcgen::RevocationReason;
    use rcgen::RevokedCertParams;
    use rcgen::SerialNumber;
    use rustls_pki_types::UnixTime;
    use time::OffsetDateTime;
    use url::Url;
    use webpki::RevocationReason as WebpkiRevocationReason;
    use x509_parser::parse_x509_crl;

    use crl::*;

    use super::*;
    use crate::server_keys::generate::Ca;
    use crate::x509::CertificateConfiguration;
    use crate::x509::DistinguishedName;
    use crate::x509::NO_SAN;

    mod crl {
        //! Minimal CertificateList datatypes, to support tests parsing an optional nextUpdate parameter.
        //! Needed because rcgen::CertificateRevocationList requires nextUpdate.
        use der::Sequence;
        use der::asn1::BitStringRef;
        use der::asn1::ObjectIdentifier;
        use der::asn1::SequenceOf;
        use der::asn1::UtcTime;

        /// `AlgorithmIdentifier ::= SEQUENCE { algorithm OBJECT IDENTIFIER }` (RFC 5280, 4.1.1.2),
        /// simplified by leaving out the OPTIONAL `parameters` field.
        #[derive(Sequence)]
        pub(super) struct AlgorithmIdentifier {
            pub(super) algorithm: ObjectIdentifier,
        }

        /// ```text
        /// TBSCertList ::= SEQUENCE {
        ///      signature               AlgorithmIdentifier,
        ///      issuer                  Name,
        ///      thisUpdate              Time }
        /// ```
        /// (RFC 5280, 5.1.2), with `version`, `nextUpdate`, `revokedCertificates` and
        /// `crlExtensions` left out, since all are OPTIONAL and `nextUpdate` is the field under test.
        #[derive(Sequence)]
        pub(super) struct TbsCertList {
            pub(super) signature: AlgorithmIdentifier,
            pub(super) issuer: SequenceOf<ObjectIdentifier, 0>,
            pub(super) this_update: UtcTime,
        }

        /// ```text
        /// CertificateList ::= SEQUENCE {
        ///      tbsCertList          TBSCertList,
        ///      signatureAlgorithm   AlgorithmIdentifier,
        ///      signatureValue       BIT STRING }
        /// ```
        /// (RFC 5280, 5.1.1).
        #[derive(Sequence)]
        pub(super) struct CertificateList<'a> {
            pub(super) tbs_cert_list: TbsCertList,
            pub(super) signature_algorithm: AlgorithmIdentifier,
            pub(super) signature_value: BitStringRef<'a>,
        }
    }

    const OID_SHA256_WITH_RSA_ENCRYPTION: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.11");

    /// Build a minimal, DER-encoded `CertificateList` (RFC 5280) whose `TBSCertList` goes
    /// straight from `thisUpdate` to the signature, omitting the optional `nextUpdate` field.
    /// rcgen always emits `nextUpdate`, so this case cannot be constructed through it.
    fn crl_der_without_next_update() -> Vec<u8> {
        let tbs_cert_list = TbsCertList {
            signature: AlgorithmIdentifier {
                algorithm: OID_SHA256_WITH_RSA_ENCRYPTION,
            },
            issuer: SequenceOf::new(), // empty Name
            this_update: UtcTime::from_unix_duration(Duration::ZERO).unwrap(),
        };

        CertificateList {
            tbs_cert_list,
            signature_algorithm: AlgorithmIdentifier {
                algorithm: OID_SHA256_WITH_RSA_ENCRYPTION,
            },
            signature_value: BitStringRef::from_bytes(&[]).unwrap(),
        }
        .to_der()
        .unwrap()
    }

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

    #[test]
    fn ttl_from_next_update_returns_remaining_duration() {
        let ca = Ca::generate_mock();
        let now = OffsetDateTime::now_utc();
        let next_update = now + Duration::from_secs(3600);
        let crl = ca.generate_crl_with_validity(vec![], now, next_update).unwrap();

        let (_, parsed) = parse_x509_crl(crl.der()).unwrap();
        let ttl = ttl_from_next_update(&parsed).unwrap();

        // Allow some slack for the time elapsed during test execution.
        assert!(ttl <= Duration::from_secs(3600));
        assert!(ttl > Duration::from_secs(3600 - 60));
    }

    #[test]
    fn ttl_from_next_update_returns_zero_when_expired() {
        let ca = Ca::generate_mock();
        let this_update = OffsetDateTime::UNIX_EPOCH;
        let next_update = this_update + Duration::from_secs(3600);
        let crl = ca.generate_crl_with_validity(vec![], this_update, next_update).unwrap();

        let (_, parsed) = parse_x509_crl(crl.der()).unwrap();
        let ttl = ttl_from_next_update(&parsed).unwrap();

        assert_eq!(ttl, Duration::ZERO);
    }

    #[test]
    fn ttl_from_next_update_returns_none_without_next_update() {
        let der = crl_der_without_next_update();
        let (_, parsed) = parse_x509_crl(&der).unwrap();

        assert!(ttl_from_next_update(&parsed).is_none());
    }

    fn empty_revocation_list() -> Vec<u8> {
        let ca = Ca::generate_mock();
        ca.generate_crl(vec![]).unwrap().der().to_vec()
    }

    #[tokio::test]
    async fn crl_provider_caches_response_for_repeated_fetches() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(200).body(empty_revocation_list());
            })
            .await;

        let url: Url = server.url("/crl.der").parse().unwrap();
        let cert = generate_cert_with_cdps(vec![url]);
        let provider = CrlProvider::new(httpmock_reqwest_client_builder().build().unwrap(), 10);

        let first = provider.crls_for_cert(&cert).await.unwrap();
        let second = provider.crls_for_cert(&cert).await.unwrap();

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);

        // Server should have been invoked once
        mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn crl_provider_without_caching_refetches_every_time() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(200).body(empty_revocation_list());
            })
            .await;

        let url: Url = server.url("/crl.der").parse().unwrap();
        let cert = generate_cert_with_cdps(vec![url]);
        let provider = CrlProvider::new_without_caching(httpmock_reqwest_client_builder().build().unwrap());

        provider.crls_for_cert(&cert).await.unwrap();
        provider.crls_for_cert(&cert).await.unwrap();

        // Server should have been invoked twice
        mock.assert_calls_async(2).await;
    }

    #[tokio::test]
    async fn crl_provider_refetches_after_ttl_expires() {
        let server = MockServer::start_async().await;
        let this_update = OffsetDateTime::now_utc();
        // Expires in 100 milliseconds
        let next_update = this_update + Duration::from_millis(100);
        let ca = Ca::generate_mock();
        let crl = ca.generate_crl_with_validity(vec![], this_update, next_update).unwrap();

        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(200).body(crl.der());
            })
            .await;

        let url: Url = server.url("/crl.der").parse().unwrap();
        let cert = generate_cert_with_cdps(vec![url]);
        let provider = CrlProvider::new(httpmock_reqwest_client_builder().build().unwrap(), 10);

        // Invoke once to initialize the cache
        provider.crls_for_cert(&cert).await.unwrap();
        mock.assert_calls_async(1).await;

        // Wait 200 milliseconds until CRL is expired
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Invoke again after expiry
        provider.crls_for_cert(&cert).await.unwrap();

        // Server should have been invoked twice
        mock.assert_calls_async(2).await;
    }

    #[tokio::test]
    async fn crl_provider_returns_http_error_on_server_failure() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(500).body("server error");
            })
            .await;

        let url: Url = server.url("/crl.der").parse().unwrap();
        let cert = generate_cert_with_cdps(vec![url]);
        let provider = CrlProvider::new(httpmock_reqwest_client_builder().build().unwrap(), 10);

        let error = provider.crls_for_cert(&cert).await.unwrap_err();

        assert!(matches!(error, CrlProviderError::Http(_)));
        mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn crl_provider_returns_parsing_error_on_invalid_der() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(200).body("not a crl");
            })
            .await;

        let url: Url = server.url("/crl.der").parse().unwrap();
        let cert = generate_cert_with_cdps(vec![url]);
        let provider = CrlProvider::new(httpmock_reqwest_client_builder().build().unwrap(), 10);

        let error = provider.crls_for_cert(&cert).await.unwrap_err();

        assert!(matches!(error, CrlProviderError::Parsing(_)));
        mock.assert_calls_async(1).await;
    }
}
