use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use chrono::DateTime;
use chrono::Utc;
use error_category::ErrorCategory;
use futures::future;
use http_utils::reqwest::bytes_with_max_response_size;
use itertools::Itertools;
use moka::Expiry;
use moka::future::Cache;
use reqwest::Client;
use url::Url;
use utils::generator::Generator;
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

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum CrlFetchError {
    #[error("HTTP error fetching CRL: {0}")]
    Http(#[source] reqwest::Error),
    #[error("CRL response exceeds maximum size of {MAX_CRL_SIZE} bytes")]
    #[category(critical)]
    TooLarge,
    #[cfg(any(test, feature = "mock"))]
    #[error("no mock CRL configured for URL: {0}")]
    MockCrlNotFound(Url),
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum CrlRetrievalError {
    #[error("failed to fetch CRL from {url}: {source}")]
    Fetch {
        url: Url,
        #[source]
        source: CrlFetchError,
    },
    #[error("failed to parse CRL from {url}: {source}")]
    Parsing {
        url: Url,
        #[source]
        source: webpki::Error,
    },
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum CertificateCrlVerificationError {
    #[error("certificate verification failed: {0}")]
    Certificate(#[source] CertificateError),
    #[error("certificate revocation verification failed: {0}")]
    Revocation(#[source] CertificateError),
    #[error("all CRL distribution points for at least one certificate failed; first failure: {source}; additional failures: {}", additional_errors.iter().join(", "))]
    #[category(pd)]
    CrlRetrieval {
        #[source]
        source: Box<CrlRetrievalError>,
        additional_errors: Vec<CrlRetrievalError>,
    },
    #[error("invalid CRL distribution point URL: {0}")]
    #[category(pd)]
    InvalidDistributionPoint(#[source] url::ParseError),
    #[error("certificate chain is empty")]
    #[category(critical)]
    EmptyChain,
    #[error("no usable CRL distribution point available for certificate")]
    #[category(critical)]
    NoCrlDistributionPoint,
}

/// TTL used for a cached CRL when its `nextUpdate` field could not be determined, to guard against caching an entry
/// indefinitely.
const FALLBACK_TTL: Duration = Duration::from_mins(5);

/// Upper bound on the cache TTL derived from a CRL's `nextUpdate` field.
const MAX_TTL: Duration = Duration::from_hours(7 * 24);

/// Upper bound on the size of a single CRL response. Real-world CRLs are typically well under 1 MB (median around
/// 90 KB among popular CAs); this is generous headroom for the closed, comparatively small WRPAC access-certificate
/// ecosystem, while still bounding memory use against a malicious or malfunctioning CRL distribution point.
const MAX_CRL_SIZE: usize = 5 * 1024 * 1024;

/// Maximum number of CRLs retained by the default verifier used by the wallet.
const DEFAULT_CACHE_CAPACITY: u64 = 100;

/// A parsed CRL together with its cache TTL.
#[derive(Debug)]
struct ParsedCrl {
    crl: CertRevocationList<'static>,
    ttl: Duration,
}

/// The result of fetching a single CRL for use in one `verify_chain` call: either served from the cache, or retrieved
/// fresh over the network this call. A fresh result's signature has not been checked yet, so it is only a candidate for
/// insertion into the cache — see `verify_chain`, which commits it after a successful verification.
#[derive(Debug)]
enum FetchedCrl {
    Cached(Arc<ParsedCrl>),
    Fresh { url: Url, fetched: Arc<ParsedCrl> },
}

impl FetchedCrl {
    fn crl(&self) -> &CertRevocationList<'static> {
        match self {
            FetchedCrl::Cached(cached) => &cached.crl,
            FetchedCrl::Fresh { fetched, .. } => &fetched.crl,
        }
    }
}

struct CrlExpiry;

impl Expiry<Url, Arc<ParsedCrl>> for CrlExpiry {
    fn expire_after_create(&self, _key: &Url, value: &Arc<ParsedCrl>, _created_at: Instant) -> Option<Duration> {
        Some(value.ttl)
    }
}

/// Retrieves a DER-encoded CRL from a distribution point.
pub trait CrlFetcher {
    async fn fetch(&self, url: &Url) -> Result<Vec<u8>, CrlFetchError>;
}

/// Retrieves CRLs over HTTP using an injected client.
#[derive(Clone, Debug)]
pub struct HttpCrlFetcher {
    client: Client,
}

impl HttpCrlFetcher {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

impl CrlFetcher for HttpCrlFetcher {
    async fn fetch(&self, url: &Url) -> Result<Vec<u8>, CrlFetchError> {
        let mut response = self
            .client
            .get(url.clone())
            .send()
            .await
            .map_err(CrlFetchError::Http)?
            .error_for_status()
            .map_err(CrlFetchError::Http)?;

        // Read the body in chunks rather than via `.bytes()`, so a CRL distribution point cannot
        // exhaust memory by returning an unbounded or never-ending response body.
        let bytes = bytes_with_max_response_size(&mut response, MAX_CRL_SIZE)
            .await
            .map_err(CrlFetchError::Http)?
            .ok_or(CrlFetchError::TooLarge)?;

        Ok(bytes.to_vec())
    }
}

/// Retrieves and caches RFC 5280 CRLs keyed by URL while verifying certificate chains.
///
/// The cache TTL for each entry is derived from the CRL's `nextUpdate` field so entries are refreshed automatically.
/// Freshly-fetched CRL candidates are only committed to the cache after the certificate chain successfully verifies;
/// cached candidates are checked again whenever `rustls-webpki` selects them for a certificate.
#[derive(Clone, Debug)]
pub struct CertificateCrlVerifier<F = HttpCrlFetcher> {
    fetcher: F,
    cache: Cache<Url, Arc<ParsedCrl>>,
}

impl CertificateCrlVerifier<HttpCrlFetcher> {
    pub fn new_with_default_cache(client: Client) -> Self {
        Self::new(client, DEFAULT_CACHE_CAPACITY)
    }

    pub fn new(client: Client, max_capacity: u64) -> Self {
        Self::new_with_fetcher(HttpCrlFetcher::new(client), max_capacity)
    }
}

impl<F> CertificateCrlVerifier<F> {
    pub fn new_with_fetcher(fetcher: F, max_capacity: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .expire_after(CrlExpiry)
            .build();
        Self { fetcher, cache }
    }
}

impl<F> CertificateCrlVerifier<F>
where
    F: CrlFetcher,
{
    /// Resolve the CRLs referenced by every certificate, either from cache or the network. URLs are deduplicated across
    /// the chain and fetched concurrently. A certificate's failed distribution point is tolerated when at least one
    /// alternative succeeds.
    async fn crls_for_chain(
        &self,
        chain: &[BorrowingCertificate],
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<Vec<FetchedCrl>, CertificateCrlVerificationError> {
        let urls_by_cert = chain
            .iter()
            .map(extract_crl_distribution_points)
            .collect::<Result<Vec<_>, _>>()
            .map_err(CertificateCrlVerificationError::InvalidDistributionPoint)?;

        if urls_by_cert.iter().any(Vec::is_empty) {
            return Err(CertificateCrlVerificationError::NoCrlDistributionPoint);
        }

        let mut seen = HashSet::new();
        let unique_urls = urls_by_cert
            .iter()
            .flatten()
            .filter(|url| seen.insert((*url).clone()))
            .cloned()
            .collect_vec();

        // Exact cache hits avoid another download, but uncached sibling URLs are retried. Since `webpki` does not
        // report which candidate it selected during an earlier successful verification, a cached sibling alone is not
        // proof that it was authoritative.
        let results = future::join_all(unique_urls.into_iter().map(|url| async move {
            let result = self.fetch_crl(url.clone(), time).await;
            (url, result)
        }))
        .await;

        // A certificate needs one authoritative complete CRL rather than every advertised URL. Successfully fetched
        // candidates are still validated by `webpki`, which rejects partial CRL forms that would require combining
        // multiple CRLs. Record the URLs that failed, then find every certificate for which all distribution points
        // failed; errors from a failed alternative are ignored when the same certificate has another working candidate.
        let error_urls = results
            .iter()
            .filter_map(|(url, result)| result.as_ref().err().map(|_| url))
            .collect::<HashSet<_>>();
        let failed_urls = urls_by_cert
            .iter()
            .filter(|urls| urls.iter().all(|url| error_urls.contains(url)))
            .flatten()
            .cloned()
            .collect::<HashSet<_>>();

        // Preserve the certificate and distribution-point order used above. This keeps both the primary error and the
        // returned CRLs deterministic, unlike iterating over the sets used only for membership checks.
        let (mut crls, mut errors) = (Vec::new(), Vec::new());
        for (url, result) in results {
            match result {
                Ok(crl) => crls.push(crl),
                Err(error) if failed_urls.contains(&url) => errors.push(error),
                Err(_) => {}
            }
        }

        let mut errors = errors.into_iter();
        if let Some(source) = errors.next() {
            return Err(CertificateCrlVerificationError::CrlRetrieval {
                source: Box::new(source),
                additional_errors: errors.collect(),
            });
        }

        Ok(crls)
    }

    /// Fetch and parse the CRL at `url`, or return the already-parsed, cached CRL if present. A freshly-fetched CRL's
    /// signature has not yet been checked, so it is not inserted into the cache here — the caller commits the candidate
    /// set via `self.cache.insert` only after successful chain verification in `verify_chain`.
    async fn fetch_crl(&self, url: Url, time: &impl Generator<DateTime<Utc>>) -> Result<FetchedCrl, CrlRetrievalError> {
        if let Some(cached) = self.cache.get(&url).await {
            return Ok(FetchedCrl::Cached(cached));
        }
        let bytes = self
            .fetcher
            .fetch(&url)
            .await
            .map_err(|source| CrlRetrievalError::Fetch {
                url: url.clone(),
                source,
            })?;

        // `rustls-webpki` parses and enforces `nextUpdate`, but does not expose
        // its value. Use `x509_parser` only as a best-effort metadata pass to
        // derive the cache TTL; `webpki` below remains authoritative for parsing
        // and verification. Fall back to a short TTL if extraction fails, and cap
        // it to impose a sane upper bound on the cache lifetime. Use the injected time so
        // cache eviction and verification use the same time source.
        let ttl = parse_x509_crl(&bytes)
            .ok()
            .and_then(|(_, crl)| ttl_from_next_update(&crl, time))
            .unwrap_or(FALLBACK_TTL)
            .min(MAX_TTL);

        let crl = parse_crl_der(&bytes).map_err(|source| CrlRetrievalError::Parsing {
            url: url.clone(),
            source,
        })?;
        let fetched = Arc::new(ParsedCrl { crl, ttl });
        Ok(FetchedCrl::Fresh { url, fetched })
    }

    /// Verify a certificate chain, checking the revocation status of every certificate in the chain against their CRLs.
    pub async fn verify_chain(
        &self,
        chain: &[BorrowingCertificate],
        trust_anchors: &TrustAnchors,
        usage: Option<CertificateUsage>,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<(), CertificateCrlVerificationError> {
        let (leaf, intermediate_certs) = chain.split_first().ok_or(CertificateCrlVerificationError::EmptyChain)?;

        // Validate the certificate path before following distribution-point URLs supplied by the certificate. This
        // prevents an untrusted certificate from turning CRL retrieval into an arbitrary network request.
        leaf.verify(usage, intermediate_certs, time, trust_anchors)
            .map_err(CertificateCrlVerificationError::Certificate)?;

        let crls = self.crls_for_chain(chain, time).await?;

        let crl_refs = crls.iter().map(FetchedCrl::crl).collect_vec();
        leaf.verify_with_crls(usage, intermediate_certs, time, trust_anchors, &crl_refs)
            .map_err(CertificateCrlVerificationError::Revocation)?;

        // Commit any freshly-fetched CRLs to the cache after successful verification.
        for fetched in crls {
            if let FetchedCrl::Fresh { url, fetched: parsed } = fetched {
                self.cache.insert(url, parsed).await;
            }
        }
        Ok(())
    }
}

/// Extract and parse all CRL distribution point URIs from the certificate's CDP extension.
/// See RFC 5280, section 4.2.1.13.
pub fn extract_crl_distribution_points(cert: &BorrowingCertificate) -> Result<Vec<Url>, url::ParseError> {
    cert.x509_certificate()
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
                    Some(Url::parse(uri))
                }
                _ => None,
            }
        })
        .collect()
}

/// Parse CRL DER bytes into a [`CertRevocationList`] ready for use with
/// [`BorrowingCertificate::verify_with_crls`].
pub(super) fn parse_crl_der(crl_der: &[u8]) -> Result<CertRevocationList<'static>, webpki::Error> {
    let owned = OwnedCertRevocationList::from_der(crl_der)?;
    Ok(CertRevocationList::from(owned))
}

/// Return remaining time until the CRL's `nextUpdate` field expires, relative to `time`.
/// Returns `None` if the CRL has no `nextUpdate`.
/// Used by callers to derive cache TTL.
fn ttl_from_next_update(crl: &CertificateRevocationList, time: &impl Generator<DateTime<Utc>>) -> Option<Duration> {
    let next_update_secs = crl.next_update()?.to_datetime().unix_timestamp();
    let now_secs = time.generate().timestamp();
    let remaining = (next_update_secs - now_secs).max(0) as u64;
    Some(Duration::from_secs(remaining))
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::collections::HashMap;
    use std::sync::LazyLock;

    use super::*;
    #[cfg(feature = "generate")]
    use crate::server_keys::generate::Ca;

    pub static MOCK_CRL_DISTRIBUTION_POINT: LazyLock<Url> =
        LazyLock::new(|| "https://example.com/crl.der".parse().unwrap());

    #[derive(Clone, Debug, Default)]
    pub struct MockCrlFetcher {
        crls: Arc<HashMap<Url, Vec<u8>>>,
    }

    impl Default for CertificateCrlVerifier<MockCrlFetcher> {
        fn default() -> Self {
            Self::new_with_fetcher(MockCrlFetcher::default(), DEFAULT_CACHE_CAPACITY)
        }
    }

    #[cfg(feature = "generate")]
    impl CertificateCrlVerifier<MockCrlFetcher> {
        pub fn new_for_ca(ca: &Ca) -> Self {
            let crl = ca.generate_crl(vec![], 1).unwrap().der().to_vec();
            Self::new_with_fetcher(
                MockCrlFetcher::new([(MOCK_CRL_DISTRIBUTION_POINT.clone(), crl)]),
                DEFAULT_CACHE_CAPACITY,
            )
        }
    }

    impl MockCrlFetcher {
        pub fn new(crls: impl IntoIterator<Item = (Url, Vec<u8>)>) -> Self {
            Self {
                crls: Arc::new(crls.into_iter().collect()),
            }
        }
    }

    impl CrlFetcher for MockCrlFetcher {
        async fn fetch(&self, url: &Url) -> Result<Vec<u8>, CrlFetchError> {
            self.crls
                .get(url)
                .cloned()
                .ok_or_else(|| CrlFetchError::MockCrlNotFound(url.clone()))
        }
    }

    impl CertificateCrlVerifier<HttpCrlFetcher> {
        pub fn new_without_caching(client: Client) -> Self {
            Self::new_with_fetcher(HttpCrlFetcher::new(client), 0)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::future::poll_fn;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::task::Poll;
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

    use super::mock::MockCrlFetcher;
    use super::*;
    use crate::server_keys::generate::Ca;
    use crate::trust_anchor::TrustAnchors;
    use crate::x509::CertificateConfiguration;
    use crate::x509::CertificateError;
    use crate::x509::DistinguishedName;
    use crate::x509::NO_SAN;

    #[derive(Clone, Debug)]
    struct ConcurrencyTrackingFetcher {
        crls: Arc<HashMap<Url, Vec<u8>>>,
        in_flight: Arc<AtomicUsize>,
        max_in_flight: Arc<AtomicUsize>,
    }

    impl ConcurrencyTrackingFetcher {
        fn new(crls: impl IntoIterator<Item = (Url, Vec<u8>)>) -> Self {
            Self {
                crls: Arc::new(crls.into_iter().collect()),
                in_flight: Arc::new(AtomicUsize::new(0)),
                max_in_flight: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn max_in_flight(&self) -> usize {
            self.max_in_flight.load(Ordering::SeqCst)
        }
    }

    impl CrlFetcher for ConcurrencyTrackingFetcher {
        async fn fetch(&self, url: &Url) -> Result<Vec<u8>, CrlFetchError> {
            let in_flight = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
            self.max_in_flight.fetch_max(in_flight, Ordering::SeqCst);

            // Yield once so all fetch futures can be polled before any of them completes.
            let mut yielded = false;
            poll_fn(|cx| {
                if yielded {
                    Poll::Ready(())
                } else {
                    yielded = true;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            })
            .await;

            self.in_flight.fetch_sub(1, Ordering::SeqCst);
            self.crls
                .get(url)
                .cloned()
                .ok_or_else(|| CrlFetchError::MockCrlNotFound(url.clone()))
        }
    }

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
        assert!(extract_crl_distribution_points(&cert).unwrap().is_empty());
    }

    #[test]
    fn single_crl_distribution_point() {
        let url: Url = "http://crl.example.com/crl.crl".parse().unwrap();
        let cert = generate_cert_with_cdps(vec![url.clone()]);
        let result = extract_crl_distribution_points(&cert).unwrap();
        assert_eq!(result, vec![url]);
    }

    #[test]
    fn multiple_crl_distribution_points() {
        let url1: Url = "http://crl.example.com/crl1.crl".parse().unwrap();
        let url2: Url = "http://crl.example.com/crl2.crl".parse().unwrap();
        let cert = generate_cert_with_cdps(vec![url1.clone(), url2.clone()]);
        let result = extract_crl_distribution_points(&cert).unwrap();
        assert_eq!(result, vec![url1, url2]);
    }

    #[test]
    fn parse_empty_crl() {
        let ca = Ca::generate_mock();
        let crl = ca.generate_crl(vec![], 1).unwrap();
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
        let crl = ca.generate_crl(vec![revoked], 1).unwrap();

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
        let crl = ca.generate_crl_with_validity(vec![], now, next_update, 1).unwrap();

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
        let crl = ca
            .generate_crl_with_validity(vec![], this_update, next_update, 1)
            .unwrap();

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
        ca.generate_crl(vec![], 1).unwrap().der().to_vec()
    }

    #[tokio::test]
    async fn verify_chain_caches_crl_after_successful_verification() {
        let server = MockServer::start_async().await;
        let url: Url = server.url("/crl.der").parse().unwrap();
        let (ca, leaf) = ca_and_leaf_with_cdps(vec![url]);
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(200).body(ca.generate_crl(vec![], 1).unwrap().der());
            })
            .await;

        let provider = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);
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
    async fn verify_chain_fetches_uncached_alternative_alongside_cached_crl() {
        let server = MockServer::start_async().await;
        let cached_url: Url = server.url("/cached.crl").parse().unwrap();
        let alternative_url: Url = server.url("/alternative.crl").parse().unwrap();
        let ca = Ca::generate_mock();
        let cached_leaf = leaf_with_cdps(&ca, vec![cached_url.clone()]);
        let leaf_with_alternatives = leaf_with_cdps(&ca, vec![cached_url, alternative_url]);
        let cached_mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/cached.crl");
                then.status(200).body(ca.generate_crl(vec![], 1).unwrap().der());
            })
            .await;
        let alternative_mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/alternative.crl");
                then.status(500);
            })
            .await;

        let provider = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);
        let trust_anchors = TrustAnchors::from(&ca);

        provider
            .verify_chain(&[cached_leaf], &trust_anchors, None, &TimeGenerator)
            .await
            .expect("certificate should verify and populate the CRL cache");
        provider
            .verify_chain(&[leaf_with_alternatives], &trust_anchors, None, &TimeGenerator)
            .await
            .expect("certificate should verify using the cached distribution point");

        cached_mock.assert_calls_async(1).await;
        alternative_mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn crl_verifier_without_caching_refetches_every_time() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(200).body(empty_revocation_list());
            })
            .await;

        let url: Url = server.url("/crl.der").parse().unwrap();
        let cert = generate_cert_with_cdps(vec![url]);
        let provider = CertificateCrlVerifier::new_without_caching(httpmock_reqwest_client_builder().build().unwrap());

        provider
            .crls_for_chain(std::slice::from_ref(&cert), &TimeGenerator)
            .await
            .unwrap();
        provider
            .crls_for_chain(std::slice::from_ref(&cert), &TimeGenerator)
            .await
            .unwrap();

        // Server should have been invoked twice
        mock.assert_calls_async(2).await;
    }

    #[tokio::test]
    async fn crl_verifier_returns_http_error_on_server_failure() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(500).body("server error");
            })
            .await;

        let url: Url = server.url("/crl.der").parse().unwrap();
        let cert = generate_cert_with_cdps(vec![url]);
        let provider = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);

        let error = provider
            .crls_for_chain(std::slice::from_ref(&cert), &TimeGenerator)
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            CertificateCrlVerificationError::CrlRetrieval {
                source,
                additional_errors,
            } if matches!(*source, CrlRetrievalError::Fetch {
                source: CrlFetchError::Http(_),
                ..
            }) && additional_errors.is_empty()
        ));
        mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn verify_chain_uses_working_alternative_distribution_point() {
        let server = MockServer::start_async().await;
        let unavailable_url: Url = server.url("/unavailable.crl").parse().unwrap();
        let available_url: Url = server.url("/available.crl").parse().unwrap();
        let (ca, leaf) = ca_and_leaf_with_cdps(vec![unavailable_url, available_url]);
        let unavailable_mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/unavailable.crl");
                then.status(500);
            })
            .await;
        let available_mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/available.crl");
                then.status(200).body(ca.generate_crl(vec![], 1).unwrap().der());
            })
            .await;

        let provider = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);
        provider
            .verify_chain(&[leaf], &TrustAnchors::from(&ca), None, &TimeGenerator)
            .await
            .expect("certificate should verify using the working distribution point");

        unavailable_mock.assert_calls_async(1).await;
        available_mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn verify_chain_fetches_crls_concurrently() {
        let leaf_crl_url: Url = "https://example.com/leaf.crl".parse().unwrap();
        let intermediate_crl_url: Url = "https://example.com/intermediate.crl".parse().unwrap();
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
                    crl_distribution_points: vec![intermediate_crl_url.clone()],
                    ..Default::default()
                },
            )
            .unwrap();
        let leaf = intermediate
            .generate_key_pair(
                DistinguishedName::create_mock("leaf"),
                CertificateConfiguration {
                    crl_distribution_points: vec![leaf_crl_url.clone()],
                    ..Default::default()
                },
                NO_SAN,
            )
            .unwrap();
        let fetcher = ConcurrencyTrackingFetcher::new([
            (
                leaf_crl_url,
                intermediate.generate_crl(vec![], 1).unwrap().der().to_vec(),
            ),
            (
                intermediate_crl_url,
                root.generate_crl(vec![], 1).unwrap().der().to_vec(),
            ),
        ]);
        let verifier = CertificateCrlVerifier::new_with_fetcher(fetcher.clone(), 10);

        verifier
            .verify_chain(
                &[
                    leaf.certificate().clone(),
                    intermediate.as_borrowing_certificate().unwrap(),
                ],
                &TrustAnchors::from(&root),
                None,
                &TimeGenerator,
            )
            .await
            .expect("certificate chain should verify");

        assert_eq!(fetcher.max_in_flight(), 2);
    }

    #[tokio::test]
    async fn verify_chain_deduplicates_repeated_distribution_point_url() {
        let server = MockServer::start_async().await;
        let url: Url = server.url("/crl.der").parse().unwrap();
        let (ca, leaf) = ca_and_leaf_with_cdps(vec![url.clone(), url]);
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(200).body(ca.generate_crl(vec![], 1).unwrap().der());
            })
            .await;

        let verifier = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);
        verifier
            .verify_chain(&[leaf], &TrustAnchors::from(&ca), None, &TimeGenerator)
            .await
            .expect("certificate should verify with a deduplicated CRL");

        mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn crl_verifier_returns_all_distribution_point_errors() {
        let server = MockServer::start_async().await;
        let unavailable_url: Url = server.url("/unavailable.crl").parse().unwrap();
        let malformed_url: Url = server.url("/malformed.crl").parse().unwrap();
        let cert = generate_cert_with_cdps(vec![unavailable_url.clone(), malformed_url.clone()]);
        let unavailable_mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/unavailable.crl");
                then.status(500);
            })
            .await;
        let malformed_mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/malformed.crl");
                then.status(200).body("not a crl");
            })
            .await;

        let provider = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);
        let error = provider
            .crls_for_chain(std::slice::from_ref(&cert), &TimeGenerator)
            .await
            .unwrap_err();

        assert_eq!(error.category(), error_category::Category::PersonalData);
        match error {
            CertificateCrlVerificationError::CrlRetrieval {
                source,
                additional_errors,
            } => {
                assert!(matches!(
                    *source,
                    CrlRetrievalError::Fetch {
                        url,
                        source: CrlFetchError::Http(_),
                    } if url == unavailable_url
                ));
                assert!(matches!(
                    additional_errors.as_slice(),
                    [CrlRetrievalError::Parsing { url, .. }] if url == &malformed_url
                ));
            }
            error => panic!("unexpected error: {error:?}"),
        }
        unavailable_mock.assert_calls_async(1).await;
        malformed_mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn crl_verifier_returns_errors_for_every_certificate_without_working_distribution_point() {
        let server = MockServer::start_async().await;
        let unavailable_url: Url = server.url("/unavailable.crl").parse().unwrap();
        let malformed_url: Url = server.url("/malformed.crl").parse().unwrap();
        let unavailable_cert = generate_cert_with_cdps(vec![unavailable_url.clone()]);
        let malformed_cert = generate_cert_with_cdps(vec![malformed_url.clone()]);
        let unavailable_mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/unavailable.crl");
                then.status(500);
            })
            .await;
        let malformed_mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/malformed.crl");
                then.status(200).body("not a crl");
            })
            .await;

        let provider = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);
        let error = provider
            .crls_for_chain(&[unavailable_cert, malformed_cert], &TimeGenerator)
            .await
            .unwrap_err();

        match error {
            CertificateCrlVerificationError::CrlRetrieval {
                source,
                additional_errors,
            } => {
                assert!(matches!(
                    *source,
                    CrlRetrievalError::Fetch {
                        url,
                        source: CrlFetchError::Http(_),
                    } if url == unavailable_url
                ));
                assert!(matches!(
                    additional_errors.as_slice(),
                    [CrlRetrievalError::Parsing { url, .. }] if url == &malformed_url
                ));
            }
            error => panic!("unexpected error: {error:?}"),
        }
        unavailable_mock.assert_calls_async(1).await;
        malformed_mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn crl_verifier_returns_parsing_error_on_invalid_der() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(200).body("not a crl");
            })
            .await;

        let url: Url = server.url("/crl.der").parse().unwrap();
        let cert = generate_cert_with_cdps(vec![url]);
        let provider = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);

        let error = provider
            .crls_for_chain(std::slice::from_ref(&cert), &TimeGenerator)
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            CertificateCrlVerificationError::CrlRetrieval {
                source,
                additional_errors,
            } if matches!(*source, CrlRetrievalError::Parsing { .. }) && additional_errors.is_empty()
        ));
        mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn crl_verifier_returns_too_large_error_when_response_exceeds_max_size() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(200).body(vec![0u8; MAX_CRL_SIZE + 1]);
            })
            .await;

        let url: Url = server.url("/crl.der").parse().unwrap();
        let cert = generate_cert_with_cdps(vec![url]);
        let provider = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);

        let error = provider
            .crls_for_chain(std::slice::from_ref(&cert), &TimeGenerator)
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            CertificateCrlVerificationError::CrlRetrieval {
                source,
                additional_errors,
            } if matches!(*source, CrlRetrievalError::Fetch {
                source: CrlFetchError::TooLarge,
                ..
            }) && additional_errors.is_empty()
        ));
        mock.assert_calls_async(1).await;
    }

    #[tokio::test]
    async fn crl_verifier_does_not_cache_malformed_crl_response() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(200).body("not a crl");
            })
            .await;

        let url: Url = server.url("/crl.der").parse().unwrap();
        let cert = generate_cert_with_cdps(vec![url]);
        let provider = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);

        provider
            .crls_for_chain(std::slice::from_ref(&cert), &TimeGenerator)
            .await
            .unwrap_err();
        provider
            .crls_for_chain(std::slice::from_ref(&cert), &TimeGenerator)
            .await
            .unwrap_err();

        // A response that fails to parse must not be cached, so it should be retried on every call.
        mock.assert_calls_async(2).await;
    }

    fn leaf_with_cdps(ca: &Ca, urls: Vec<Url>) -> BorrowingCertificate {
        let config = CertificateConfiguration {
            crl_distribution_points: urls,
            ..Default::default()
        };
        let leaf = ca
            .generate_key_pair(DistinguishedName::create_mock("leaf"), config, NO_SAN)
            .unwrap();
        leaf.certificate().clone()
    }

    /// Generate a CA and a leaf certificate signed by it, with the given CRL distribution points.
    fn ca_and_leaf_with_cdps(urls: Vec<Url>) -> (Ca, BorrowingCertificate) {
        let ca = Ca::generate_mock();
        let leaf = leaf_with_cdps(&ca, urls);
        (ca, leaf)
    }

    #[tokio::test]
    async fn mock_fetcher_exercises_crl_validation() {
        let url: Url = "https://example.com/crl.der".parse().unwrap();
        let (ca, leaf) = ca_and_leaf_with_cdps(vec![url.clone()]);
        let crl = ca.generate_crl(vec![], 1).unwrap().der().to_vec();
        let verifier = CertificateCrlVerifier::new_with_fetcher(MockCrlFetcher::new([(url, crl)]), 10);

        verifier
            .verify_chain(&[leaf], &TrustAnchors::from(&ca), None, &TimeGenerator)
            .await
            .expect("certificate should verify using the issuer-signed mock CRL");
    }

    #[tokio::test]
    async fn verify_chain_succeeds_for_non_revoked_certificate() {
        let server = MockServer::start_async().await;
        let url: Url = server.url("/crl.der").parse().unwrap();
        let (ca, leaf) = ca_and_leaf_with_cdps(vec![url]);
        server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(200).body(ca.generate_crl(vec![], 1).unwrap().der());
            })
            .await;

        let provider = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);

        provider
            .verify_chain(&[leaf], &TrustAnchors::from(&ca), None, &TimeGenerator)
            .await
            .expect("certificate should verify");
    }

    #[tokio::test]
    async fn verify_chain_does_not_fetch_crl_for_untrusted_certificate() {
        let server = MockServer::start_async().await;
        let url: Url = server.url("/crl.der").parse().unwrap();
        let (_untrusted_ca, leaf) = ca_and_leaf_with_cdps(vec![url]);
        let trusted_ca = Ca::generate_mock();
        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/crl.der");
                then.status(200).body("request should not be made");
            })
            .await;

        let provider = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);
        provider
            .verify_chain(&[leaf], &TrustAnchors::from(&trusted_ca), None, &TimeGenerator)
            .await
            .expect_err("untrusted certificate should fail before CRL retrieval");

        mock.assert_calls_async(0).await;
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
                then.status(200).body(ca.generate_crl(vec![revoked], 1).unwrap().der());
            })
            .await;

        let provider = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);
        let trust_anchors = TrustAnchors::from(&ca);

        let error = provider
            .verify_chain(std::slice::from_ref(&leaf), &trust_anchors, None, &TimeGenerator)
            .await
            .expect_err("revoked certificate should fail verification");
        assert!(matches!(
            error,
            CertificateCrlVerificationError::Revocation(CertificateError::Verification(error))
                if matches!(*error, webpki::Error::CertRevoked)
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
                then.status(200)
                    .body(root.generate_crl(vec![revoked], 1).unwrap().der());
            })
            .await;
        server
            .mock_async(|when, then| {
                when.method(GET).path("/intermediate.crl");
                then.status(200)
                    .body(intermediate.generate_crl(vec![], 1).unwrap().der());
            })
            .await;

        // Test Subject
        let provider = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);

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
            CertificateCrlVerificationError::Revocation(CertificateError::Verification(error))
                if matches!(*error, webpki::Error::CertRevoked)
        ));
    }

    #[tokio::test]
    async fn verify_chain_fails_when_no_crl_distribution_point_is_present() {
        let (ca, leaf) = ca_and_leaf_with_cdps(vec![]);
        let provider = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);

        let error = provider
            .verify_chain(&[leaf], &TrustAnchors::from(&ca), None, &TimeGenerator)
            .await
            .expect_err("certificate without a CDP extension should fail verification");
        assert!(matches!(error, CertificateCrlVerificationError::NoCrlDistributionPoint));
    }

    #[tokio::test]
    async fn verify_chain_requires_crl_for_each_certificate() {
        let server = MockServer::start_async().await;
        let leaf_crl_url: Url = server.url("/leaf.crl").parse().unwrap();
        let root = Ca::generate_with_intermediate_count(
            DistinguishedName::create_mock("root"),
            CertificateConfiguration::default(),
            1,
        )
        .unwrap();
        let intermediate = root
            .generate_intermediate(
                DistinguishedName::create_mock("intermediate"),
                CertificateConfiguration::default(),
            )
            .unwrap();
        let leaf = intermediate
            .generate_key_pair(
                DistinguishedName::create_mock("leaf"),
                CertificateConfiguration {
                    crl_distribution_points: vec![leaf_crl_url],
                    ..Default::default()
                },
                NO_SAN,
            )
            .unwrap();
        server
            .mock_async(|when, then| {
                when.method(GET).path("/leaf.crl");
                then.status(200)
                    .body(intermediate.generate_crl(vec![], 1).unwrap().der());
            })
            .await;

        let provider = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);
        let error = provider
            .verify_chain(
                &[
                    leaf.certificate().clone(),
                    intermediate.as_borrowing_certificate().unwrap(),
                ],
                &TrustAnchors::from(&root),
                None,
                &TimeGenerator,
            )
            .await
            .expect_err("every certificate in the chain should require a CRL");

        assert!(matches!(error, CertificateCrlVerificationError::NoCrlDistributionPoint));
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
                    ca.generate_crl_with_validity(vec![], this_update, next_update, 1)
                        .unwrap()
                        .der(),
                );
            })
            .await;

        let provider = CertificateCrlVerifier::new_without_caching(httpmock_reqwest_client_builder().build().unwrap());

        let error = provider
            .verify_chain(&[leaf], &TrustAnchors::from(&ca), None, &TimeGenerator)
            .await
            .expect_err("expired CRL should fail verification");
        assert!(matches!(
            error,
            CertificateCrlVerificationError::Revocation(CertificateError::Verification(error))
                if matches!(*error, webpki::Error::CrlExpired { .. })
        ));
    }

    #[tokio::test]
    async fn verify_chain_fails_for_empty_chain() {
        let ca = Ca::generate_mock();
        let provider = CertificateCrlVerifier::new(httpmock_reqwest_client_builder().build().unwrap(), 10);

        let error = provider
            .verify_chain(&[], &TrustAnchors::from(&ca), None, &TimeGenerator)
            .await
            .expect_err("empty chain should fail verification");
        assert!(matches!(error, CertificateCrlVerificationError::EmptyChain));
    }
}
