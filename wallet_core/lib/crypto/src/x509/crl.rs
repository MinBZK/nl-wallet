use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use chrono::DateTime;
use chrono::Utc;
use itertools::Itertools;
use moka::Expiry;
use moka::future::Cache;
use reqwest::Client;
use url::Url;
use utils::generator::Generator;
use utils::vec_at_least::VecNonEmpty;
use webpki::CertRevocationList;
use webpki::OwnedCertRevocationList;
use x509_parser::extensions::DistributionPointName;
use x509_parser::extensions::GeneralName;
use x509_parser::extensions::ParsedExtension;
use x509_parser::parse_x509_crl;
use x509_parser::revocation_list::CertificateRevocationList;

use crate::trust_anchor::TrustAnchors;
use crate::x509::BorrowingCertificate;
use crate::x509::CertificateError;
use crate::x509::CertificateUsage;

#[derive(Debug, thiserror::Error)]
pub enum CrlProviderError {
    #[error("HTTP error fetching CRL: {0}")]
    Http(#[source] reqwest::Error),
    #[error("CRL parsing error: {0}")]
    Parsing(#[source] webpki::Error),
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[source] url::ParseError),
    #[error("certificate chain is empty")]
    EmptyChain,
    #[error("no usable CRL distribution point available for certificate")]
    NoCrlDistributionPoint,
    #[error("certificate verification failed: {0}")]
    Verification(#[source] Box<CertificateError>),
}

/// TTL used for a cached CRL when its `nextUpdate` field could not be determined, to guard against caching an entry
/// indefinitely.
const FALLBACK_TTL: Duration = Duration::from_mins(5);

/// Upper bound on the cache TTL derived from a CRL's `nextUpdate` field.
const MAX_TTL: Duration = Duration::from_hours(7 * 24);

/// A parsed CRL together with its cache TTL.
#[derive(Debug)]
pub struct CachedCrl {
    crl: CertRevocationList<'static>,
    ttl: Duration,
}

/// The result of fetching a single CRL for use in one `verify_chain` call: either served from the cache, or retrieved
/// fresh over the network this call. A fresh result's signature has not been checked yet, so it is only a candidate for
/// insertion into the cache — see `verify_chain`, which commits it after a successful verification.
#[derive(Debug)]
pub enum FetchedCrl {
    Cached(Arc<CachedCrl>),
    Fresh { url: Url, fetched: Arc<CachedCrl> },
}

impl FetchedCrl {
    pub(super) fn crl(&self) -> &CertRevocationList<'static> {
        match self {
            FetchedCrl::Cached(cached) => &cached.crl,
            FetchedCrl::Fresh { fetched, .. } => &fetched.crl,
        }
    }
}

struct CrlExpiry;

impl Expiry<Url, Arc<CachedCrl>> for CrlExpiry {
    fn expire_after_create(&self, _key: &Url, value: &Arc<CachedCrl>, _created_at: Instant) -> Option<Duration> {
        Some(value.ttl)
    }
}

/// Downloads and caches RFC 5280 CRLs keyed by URL.
///
/// The cache TTL for each entry is derived from the CRL's `nextUpdate` field so entries are refreshed automatically
/// when the CRL expires. A freshly-fetched CRL is only committed to the cache once it has been used in a successful
/// `verify_chain` call (i.e. its signature has been checked by `rustls-webpki`).
pub struct CrlProvider {
    client: Client,
    cache: Cache<Url, Arc<CachedCrl>>,
}

impl CrlProvider {
    pub fn new(client: Client, max_capacity: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .expire_after(CrlExpiry)
            .build();
        Self { client, cache }
    }

    /// Verify a certificate chain, checking the revocation status of every certificate in the chain against their CRLs.
    pub async fn verify_chain(
        &self,
        chain: &[BorrowingCertificate],
        trust_anchors: &TrustAnchors,
        usage: Option<CertificateUsage>,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<(), CrlProviderError> {
        let (leaf, intermediate_certs) = chain.split_first().ok_or(CrlProviderError::EmptyChain)?;

        let mut crls = Vec::new();
        for cert in chain {
            crls.extend(self.crls_for_cert(cert, time).await?);
        }

        if crls.is_empty() {
            return Err(CrlProviderError::NoCrlDistributionPoint);
        }

        // Verify the whole certificate chain before storing fetched CRLs in cache. This is needed since we cannot
        // directly and easily verify the signature of a CRL using `rustls-webpki`.
        leaf.verify(usage, intermediate_certs, time, trust_anchors, Some(crls.as_slice()))
            .map_err(|error| CrlProviderError::Verification(Box::new(error)))?;

        // Commit any freshly-fetched CRLs to the cache after successful verification.
        for fetched in crls {
            if let FetchedCrl::Fresh { url, fetched: cached } = fetched {
                self.cache.insert(url, cached).await;
            }
        }
        Ok(())
    }

    /// Fetch all CRLs referenced in the certificate's CDP extension, either from cache or the network.
    pub async fn crls_for_cert(
        &self,
        cert: &BorrowingCertificate,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<Vec<FetchedCrl>, CrlProviderError> {
        let urls = extract_crl_distribution_points(cert);
        let mut crls = Vec::new();
        for url in urls.iter().flatten() {
            let fetched = self
                .fetch_crl(url.parse().map_err(CrlProviderError::InvalidUrl)?, time)
                .await?;
            crls.push(fetched);
        }
        Ok(crls)
    }

    /// Fetch and parse the CRL at `url`, or return the already-parsed, cached CRL if present. A freshly-fetched CRL's
    /// signature has not yet been checked, so it is not inserted into the cache here — the caller commits it via
    /// `self.cache.insert` only after successfully using it in `verify_chain`.
    async fn fetch_crl(&self, url: Url, time: &impl Generator<DateTime<Utc>>) -> Result<FetchedCrl, CrlProviderError> {
        if let Some(cached) = self.cache.get(&url).await {
            return Ok(FetchedCrl::Cached(cached));
        }
        let bytes = self
            .client
            .get(url.clone())
            .send()
            .await
            .map_err(CrlProviderError::Http)?
            .error_for_status()
            .map_err(CrlProviderError::Http)?
            .bytes()
            .await
            .map_err(CrlProviderError::Http)?;

        // Extract TTL using `x509_parser`, falling back to a short retry TTL rather than caching
        // this entry indefinitely if `nextUpdate` couldn't be determined. Uses the same injected
        // `time` as verification, so cache eviction and expiry enforcement agree on "now".
        // Capped at `MAX_TTL`, since `nextUpdate` comes from the not-yet-verified CRL itself.
        let ttl = parse_x509_crl(&bytes)
            .ok()
            .and_then(|(_, crl)| ttl_from_next_update(&crl, time))
            .unwrap_or(FALLBACK_TTL)
            .min(MAX_TTL);

        let crl = parse_crl_der(&bytes).map_err(CrlProviderError::Parsing)?;
        let fetched = Arc::new(CachedCrl { crl, ttl });
        Ok(FetchedCrl::Fresh { url, fetched })
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

/// Return remaining time until the CRL's `nextUpdate` field expires, relative to `time`.
/// Returns `None` if the CRL has no `nextUpdate`.
/// Used by callers to derive cache TTL.
pub fn ttl_from_next_update(crl: &CertificateRevocationList, time: &impl Generator<DateTime<Utc>>) -> Option<Duration> {
    let next_update_secs = crl.next_update()?.to_datetime().unix_timestamp();
    let now_secs = time.generate().timestamp();
    let remaining = (next_update_secs - now_secs).max(0) as u64;
    Some(Duration::from_secs(remaining))
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use super::*;

    impl CrlProvider {
        pub fn new_without_caching(client: Client) -> Self {
            Self {
                client,
                cache: Cache::builder().max_capacity(0).build(),
            }
        }
    }

    impl FetchedCrl {
        /// Wrap an already-parsed CRL for use in `BorrowingCertificate::verify` without going through
        /// `CrlProvider` (the TTL is irrelevant here, since it's only used by `CrlProvider`'s cache).
        #[cfg(test)]
        pub(crate) fn new_for_test(crl: CertRevocationList<'static>) -> Self {
            FetchedCrl::Cached(Arc::new(CachedCrl {
                crl,
                ttl: Duration::ZERO,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crl::*;
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
    use utils::generator::TimeGenerator;
    use utils::generator::mock::MockTimeGenerator;
    use webpki::RevocationReason as WebpkiRevocationReason;
    use x509_parser::parse_x509_crl;

    use super::*;
    use crate::server_keys::generate::Ca;
    use crate::trust_anchor::TrustAnchors;
    use crate::x509::CertificateConfiguration;
    use crate::x509::CertificateError;
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
        let now_secs = 1_700_000_000i64;
        let now = OffsetDateTime::from_unix_timestamp(now_secs).unwrap();
        let next_update = now + Duration::from_secs(3600);
        let crl = ca.generate_crl_with_validity(vec![], now, next_update).unwrap();

        let (_, parsed) = parse_x509_crl(crl.der()).unwrap();
        let time = MockTimeGenerator::new(DateTime::from_timestamp(now_secs, 0).unwrap());
        let ttl = ttl_from_next_update(&parsed, &time).unwrap();

        assert_eq!(ttl, Duration::from_secs(3600));
    }

    #[test]
    fn ttl_from_next_update_returns_zero_when_expired() {
        let ca = Ca::generate_mock();
        let this_update_secs = 0i64;
        let this_update = OffsetDateTime::UNIX_EPOCH;
        let next_update = this_update + Duration::from_secs(3600);
        let crl = ca.generate_crl_with_validity(vec![], this_update, next_update).unwrap();

        let (_, parsed) = parse_x509_crl(crl.der()).unwrap();
        // "Now" is well past `next_update`.
        let mock_now_secs = this_update_secs + 7200;
        let time = MockTimeGenerator::new(DateTime::from_timestamp(mock_now_secs, 0).unwrap());
        let ttl = ttl_from_next_update(&parsed, &time).unwrap();

        assert_eq!(ttl, Duration::ZERO);
    }

    #[test]
    fn ttl_from_next_update_returns_none_without_next_update() {
        let der = crl_der_without_next_update();
        let (_, parsed) = parse_x509_crl(&der).unwrap();

        assert!(ttl_from_next_update(&parsed, &TimeGenerator).is_none());
    }

    fn empty_revocation_list() -> Vec<u8> {
        let ca = Ca::generate_mock();
        ca.generate_crl(vec![]).unwrap().der().to_vec()
    }

    #[tokio::test]
    async fn verify_chain_caches_crl_after_successful_verification() {
        let server = MockServer::start_async().await;
        let url: Url = server.url("/crl.der").parse().unwrap();
        let (ca, leaf) = ca_and_leaf_with_cdps(vec![url]);
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(200).body(ca.generate_crl(vec![]).unwrap().der());
            })
            .await;

        let provider = CrlProvider::new(httpmock_reqwest_client_builder().build().unwrap(), 10);
        let trust_anchors = TrustAnchors::from(&ca);

        provider
            .verify_chain(std::slice::from_ref(&leaf), &trust_anchors, None, &TimeGenerator)
            .await
            .expect("certificate should verify");
        provider
            .verify_chain(&[leaf], &trust_anchors, None, &TimeGenerator)
            .await
            .expect("certificate should verify again, served from cache");

        // The second call's CRL should have come from the cache, committed after the first
        // call's successful verification, so the server should have been invoked only once.
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

        provider.crls_for_cert(&cert, &TimeGenerator).await.unwrap();
        provider.crls_for_cert(&cert, &TimeGenerator).await.unwrap();

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

        let error = provider.crls_for_cert(&cert, &TimeGenerator).await.unwrap_err();

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

        let error = provider.crls_for_cert(&cert, &TimeGenerator).await.unwrap_err();

        assert!(matches!(error, CrlProviderError::Parsing(_)));
        mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn crl_provider_does_not_cache_malformed_crl_response() {
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

        provider.crls_for_cert(&cert, &TimeGenerator).await.unwrap_err();
        provider.crls_for_cert(&cert, &TimeGenerator).await.unwrap_err();

        // A response that fails to parse must not be cached, so it should be retried on every call.
        mock.assert_calls_async(2).await;
    }

    /// Generate a CA and a leaf certificate signed by it, with the given CRL distribution points.
    fn ca_and_leaf_with_cdps(urls: Vec<Url>) -> (Ca, BorrowingCertificate) {
        let ca = Ca::generate_mock();
        let config = CertificateConfiguration {
            crl_distribution_points: urls,
            ..Default::default()
        };
        let leaf = ca
            .generate_key_pair(DistinguishedName::create_mock("leaf"), config, NO_SAN)
            .unwrap();
        let certificate = leaf.certificate().clone();
        (ca, certificate)
    }

    #[tokio::test]
    async fn verify_chain_succeeds_for_non_revoked_certificate() {
        let server = MockServer::start_async().await;
        let url: Url = server.url("/crl.der").parse().unwrap();
        let (ca, leaf) = ca_and_leaf_with_cdps(vec![url]);
        server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(200).body(ca.generate_crl(vec![]).unwrap().der());
            })
            .await;

        let provider = CrlProvider::new(httpmock_reqwest_client_builder().build().unwrap(), 10);

        provider
            .verify_chain(&[leaf], &TrustAnchors::from(&ca), None, &TimeGenerator)
            .await
            .expect("certificate should verify");
    }

    #[tokio::test]
    async fn verify_chain_fails_for_revoked_certificate() {
        let server = MockServer::start_async().await;
        let url: Url = server.url("/crl.der").parse().unwrap();
        let (ca, leaf) = ca_and_leaf_with_cdps(vec![url]);

        let serial = leaf.x509_certificate().tbs_certificate.raw_serial().to_vec();
        let revoked = RevokedCertParams {
            serial_number: SerialNumber::from_slice(&serial),
            revocation_time: OffsetDateTime::now_utc(),
            reason_code: Some(RevocationReason::KeyCompromise),
            invalidity_date: None,
        };
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(200).body(ca.generate_crl(vec![revoked]).unwrap().der());
            })
            .await;

        let provider = CrlProvider::new(httpmock_reqwest_client_builder().build().unwrap(), 10);
        let trust_anchors = TrustAnchors::from(&ca);

        let error = provider
            .verify_chain(std::slice::from_ref(&leaf), &trust_anchors, None, &TimeGenerator)
            .await
            .expect_err("revoked certificate should fail verification");
        assert!(matches!(
            error,
            CrlProviderError::Verification(error)
                if matches!(&*error, CertificateError::Verification(error) if matches!(**error, webpki::Error::CertRevoked))
        ));

        // A CRL used in a failed verification must not be cached: a repeated call should
        // refetch it from the network rather than serve it from cache.
        provider
            .verify_chain(&[leaf], &trust_anchors, None, &TimeGenerator)
            .await
            .expect_err("revoked certificate should still fail verification on retry");
        mock.assert_calls_async(2).await;
    }

    #[tokio::test]
    async fn verify_chain_fails_for_revoked_intermediate_certificate() {
        let server = MockServer::start_async().await;
        let root_crl_url: Url = server.url("/root.crl").parse().unwrap();
        let intermediate_crl_url: Url = server.url("/intermediate.crl").parse().unwrap();

        // Create root, intermediate and leaf key pairs
        let root = Ca::generate_with_intermediate_count(
            DistinguishedName::create_mock("root"),
            CertificateConfiguration::default(),
            1,
        )
        .unwrap();
        let intermediate = root
            .generate_intermediate(
                DistinguishedName::create_mock("intermediate"),
                CertificateConfiguration {
                    crl_distribution_points: vec![root_crl_url.clone()],
                    ..Default::default()
                },
            )
            .unwrap();
        let leaf = intermediate
            .generate_key_pair(
                DistinguishedName::create_mock("leaf"),
                CertificateConfiguration {
                    crl_distribution_points: vec![intermediate_crl_url.clone()],
                    ..Default::default()
                },
                NO_SAN,
            )
            .unwrap();

        // Setup CRL with revoked intermediate
        let intermediate_cert = intermediate.as_borrowing_certificate().unwrap();
        let intermediate_serial = intermediate_cert
            .x509_certificate()
            .tbs_certificate
            .raw_serial()
            .to_vec();
        let revoked = RevokedCertParams {
            serial_number: SerialNumber::from_slice(&intermediate_serial),
            revocation_time: OffsetDateTime::now_utc(),
            reason_code: Some(RevocationReason::KeyCompromise),
            invalidity_date: None,
        };

        // Setup MockServer
        server
            .mock_async(|when, then| {
                when.method(GET).path("/root.crl");
                then.status(200).body(root.generate_crl(vec![revoked]).unwrap().der());
            })
            .await;
        server
            .mock_async(|when, then| {
                when.method(GET).path("/intermediate.crl");
                then.status(200).body(intermediate.generate_crl(vec![]).unwrap().der());
            })
            .await;

        // Test Subject
        let provider = CrlProvider::new(httpmock_reqwest_client_builder().build().unwrap(), 10);

        // Verification should fail because of revoked intermediate
        let error = provider
            .verify_chain(
                &[leaf.certificate().clone(), intermediate_cert],
                &TrustAnchors::from(&root),
                None,
                &TimeGenerator,
            )
            .await
            .expect_err("chain with a revoked intermediate certificate should fail verification");
        assert!(matches!(
            error,
            CrlProviderError::Verification(error)
                if matches!(&*error, CertificateError::Verification(error) if matches!(**error, webpki::Error::CertRevoked))
        ));
    }

    #[tokio::test]
    async fn verify_chain_fails_when_no_crl_distribution_point_is_present() {
        let (ca, leaf) = ca_and_leaf_with_cdps(vec![]);
        let provider = CrlProvider::new(httpmock_reqwest_client_builder().build().unwrap(), 10);

        let error = provider
            .verify_chain(&[leaf], &TrustAnchors::from(&ca), None, &TimeGenerator)
            .await
            .expect_err("certificate without a CDP extension should fail verification");
        assert!(matches!(error, CrlProviderError::NoCrlDistributionPoint));
    }

    #[tokio::test]
    async fn verify_chain_fails_for_expired_crl() {
        let server = MockServer::start_async().await;
        let url: Url = server.url("/crl.der").parse().unwrap();
        let (ca, leaf) = ca_and_leaf_with_cdps(vec![url]);

        let this_update = OffsetDateTime::now_utc() - Duration::from_secs(7200);
        let next_update = this_update + Duration::from_secs(3600);
        server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(200).body(
                    ca.generate_crl_with_validity(vec![], this_update, next_update)
                        .unwrap()
                        .der(),
                );
            })
            .await;

        let provider = CrlProvider::new_without_caching(httpmock_reqwest_client_builder().build().unwrap());

        let error = provider
            .verify_chain(&[leaf], &TrustAnchors::from(&ca), None, &TimeGenerator)
            .await
            .expect_err("expired CRL should fail verification");
        assert!(matches!(
            error,
            CrlProviderError::Verification(error)
                if matches!(&*error, CertificateError::Verification(error) if matches!(**error, webpki::Error::CrlExpired { .. }))
        ));
    }

    #[tokio::test]
    async fn verify_chain_fails_for_empty_chain() {
        let ca = Ca::generate_mock();
        let provider = CrlProvider::new(httpmock_reqwest_client_builder().build().unwrap(), 10);

        let error = provider
            .verify_chain(&[], &TrustAnchors::from(&ca), None, &TimeGenerator)
            .await
            .expect_err("empty chain should fail verification");
        assert!(matches!(error, CrlProviderError::EmptyChain));
    }
}
