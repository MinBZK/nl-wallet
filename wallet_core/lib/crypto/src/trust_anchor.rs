use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;

use derive_more::Debug;
use indexmap::IndexSet;
use itertools::Itertools;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::TrustAnchor;
use webpki::Error;
use webpki::anchor_from_trusted_cert;
use yoke::Yoke;
use yoke::Yokeable;

use crate::x509::BorrowingCertificate;
use crate::x509::CertificateError;

#[derive(Yokeable, Debug, Clone)]
struct ParsedTrustAnchor<'a> {
    trust_anchor: TrustAnchor<'a>,
}

type YokedTrustAnchor = Yoke<ParsedTrustAnchor<'static>, Arc<CertificateDer<'static>>>;

/// The main struct for working with trust anchors. It represents the following type:
///
/// - rustls_pki_types::TrustAnchor
///
/// It can be constructed using the `from_der` method. It is parsed on construction as a borrowed type.
#[derive(Debug, Clone)]
pub struct BorrowingTrustAnchor(YokedTrustAnchor);

impl BorrowingTrustAnchor {
    pub fn from_der(der_bytes: impl Into<Vec<u8>>) -> Result<Self, Error> {
        let certificate_der = CertificateDer::from(der_bytes.into());
        let yoke = Yoke::try_attach_to_cart(Arc::from(certificate_der), |cert| {
            let trust_anchor = anchor_from_trusted_cert(cert)?;
            Ok(ParsedTrustAnchor { trust_anchor })
        })?;

        Ok(BorrowingTrustAnchor(yoke))
    }

    pub fn as_trust_anchor(&self) -> &TrustAnchor<'_> {
        &self.0.get().trust_anchor
    }

    pub fn to_owned_trust_anchor(&self) -> TrustAnchor<'static> {
        self.0.get().trust_anchor.to_owned()
    }
}

impl Hash for BorrowingTrustAnchor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.backing_cart().hash(state);
    }
}

impl PartialEq for BorrowingTrustAnchor {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for BorrowingTrustAnchor {}

impl AsRef<[u8]> for BorrowingTrustAnchor {
    fn as_ref(&self) -> &[u8] {
        self.0.backing_cart().as_ref()
    }
}

impl TryFrom<Vec<u8>> for BorrowingTrustAnchor {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        BorrowingTrustAnchor::from_der(value)
    }
}

impl From<BorrowingTrustAnchor> for Vec<u8> {
    fn from(value: BorrowingTrustAnchor) -> Self {
        value.as_ref().to_vec()
    }
}

pub struct TrustAnchors {
    certificates: IndexSet<BorrowingCertificate>,
    trust_anchors: Vec<TrustAnchor<'static>>,
}

impl TrustAnchors {
    pub fn trust_anchors(&self) -> &[TrustAnchor<'static>] {
        self.trust_anchors.as_slice()
    }

    pub fn certificates(&self) -> &IndexSet<BorrowingCertificate> {
        &self.certificates
    }
}

impl TryFrom<Vec<Vec<u8>>> for TrustAnchors {
    type Error = CertificateError;

    fn try_from(input: Vec<Vec<u8>>) -> Result<Self, Self::Error> {
        let certificates: IndexSet<BorrowingCertificate> =
            input.into_iter().map(BorrowingCertificate::from_der).try_collect()?;

        let trust_anchors: Vec<_> = certificates
            .iter()
            .map(BorrowingCertificate::as_der)
            .map(webpki::anchor_from_trusted_cert)
            .map_ok(|ta| ta.to_owned())
            .try_collect()
            .map_err(|e| CertificateError::CertificateParsing(Box::new(e)))?;

        let result = Self {
            certificates,
            trust_anchors,
        };

        Ok(result)
    }
}

impl From<TrustAnchors> for Vec<Vec<u8>> {
    fn from(value: TrustAnchors) -> Self {
        value.certificates.into_iter().map(|c| c.to_vec()).collect()
    }
}

impl TryFrom<Vec<BorrowingTrustAnchor>> for TrustAnchors {
    type Error = CertificateError;

    fn try_from(trust_anchors: Vec<BorrowingTrustAnchor>) -> Result<Self, Self::Error> {
        let certificates: IndexSet<BorrowingCertificate> = trust_anchors
            .iter()
            .map(|ta| BorrowingCertificate::from_der(ta.as_ref()))
            .try_collect()?;

        let owned_anchors = trust_anchors.iter().map(|ta| ta.to_owned_trust_anchor()).collect();

        Ok(Self {
            certificates,
            trust_anchors: owned_anchors,
        })
    }
}
