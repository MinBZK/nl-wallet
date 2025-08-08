use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;

use derive_more::Debug;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::TrustAnchor;
use webpki::Error;
use webpki::anchor_from_trusted_cert;
use yoke::Yoke;
use yoke::Yokeable;

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
