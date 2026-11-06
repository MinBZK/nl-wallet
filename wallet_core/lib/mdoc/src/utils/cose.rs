//! mdoc-specific COSE helpers.

use std::result::Result;

pub use ::cose::COSE_X5CHAIN_HEADER_LABEL;
pub use ::cose::Cose;
pub use ::cose::CoseError;
pub use ::cose::CoseKey;
pub use ::cose::TypedCose;
use coset::CoseMac0;
use coset::CoseMac0Builder;
use coset::CoseSign1;
use coset::CoseSign1Builder;
use coset::Header;
use coset::HeaderBuilder;
use coset::ProtectedHeader;
use coset::SignatureContext;
use coset::iana;
use coset::sig_structure_data;
use crypto::keys::CredentialEcdsaKey;
use crypto::wscd::DisclosureWscd;
use error_category::ErrorCategory;

fn signatures_data_and_header(payloads: &[&[u8]]) -> (Vec<Vec<u8>>, ProtectedHeader) {
    let protected_header = ProtectedHeader {
        original_data: None,
        header: HeaderBuilder::new().algorithm(iana::Algorithm::ES256).build(),
    };

    let signatures_data = payloads
        .iter()
        .map(|payload| {
            sig_structure_data(
                SignatureContext::CoseSign1,
                protected_header.clone(),
                None,
                &[],
                payload,
            )
        })
        .collect();

    (signatures_data, protected_header)
}

pub async fn sign_coses_without_payload<K: CredentialEcdsaKey>(
    keys_and_challenges: Vec<(K, &[u8])>,
    wscd: &impl DisclosureWscd<Key = K>,
    unprotected_header: Header,
) -> Result<Vec<CoseSign1>, CoseError> {
    let (keys, challenges): (Vec<_>, Vec<_>) = keys_and_challenges.into_iter().unzip();
    let (signatures_data, protected_header) = signatures_data_and_header(&challenges);

    let keys_and_signature_data = keys
        .iter()
        .zip(signatures_data)
        .map(|(key, signature_data)| (signature_data, vec![key]))
        .collect::<Vec<_>>();

    let result = wscd
        .sign(keys_and_signature_data)
        .await
        .map_err(|error| CoseError::Signing(error.into()))?;

    // Make sure we received the correct number of signatures.
    if result.len() != challenges.len() {
        return Err(CoseError::IncorrectSignatureCount {
            expected: challenges.len(),
            received: result.len(),
        });
    }

    let signed = result
        .into_iter()
        .map(|signature| {
            Ok(CoseSign1 {
                signature: signature.first().ok_or(CoseError::SignatureMissing)?.to_vec(),
                payload: None,
                protected: protected_header.clone(),
                unprotected: unprotected_header.clone(),
            })
        })
        .collect::<Result<Vec<_>, CoseError>>()?;

    Ok(signed)
}

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(pd)]
pub enum KeysError {
    #[error("key generation error: {0}")]
    KeyGeneration(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub trait ClonePayload {
    fn clone_with_payload(&self, bytes: Vec<u8>) -> Self;
    fn clone_without_payload(&self) -> Self;
}

impl<C, T> ClonePayload for TypedCose<C, T>
where
    C: ClonePayload + Cose,
{
    fn clone_with_payload(&self, bytes: Vec<u8>) -> Self {
        self.as_ref().clone_with_payload(bytes).into()
    }

    fn clone_without_payload(&self) -> Self {
        self.as_ref().clone_without_payload().into()
    }
}

impl ClonePayload for CoseSign1 {
    fn clone_with_payload(&self, bytes: Vec<u8>) -> Self {
        CoseSign1Builder::new()
            .signature(self.signature.clone())
            .protected(self.protected.header.clone())
            .unprotected(self.unprotected.clone())
            .payload(bytes)
            .build()
    }

    fn clone_without_payload(&self) -> Self {
        CoseSign1Builder::new()
            .signature(self.signature.clone())
            .protected(self.protected.header.clone())
            .unprotected(self.unprotected.clone())
            .build()
    }
}

impl ClonePayload for CoseMac0 {
    fn clone_with_payload(&self, bytes: Vec<u8>) -> Self {
        CoseMac0Builder::new()
            .tag(self.tag.clone())
            .protected(self.protected.header.clone())
            .unprotected(self.unprotected.clone())
            .payload(bytes)
            .build()
    }

    fn clone_without_payload(&self) -> Self {
        CoseMac0Builder::new()
            .tag(self.tag.clone())
            .protected(self.protected.header.clone())
            .unprotected(self.unprotected.clone())
            .build()
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use coset::Header;
    use crypto::keys::PublicKey;
    use crypto::mock_remote::DisclosureMockRemoteWscd;
    use crypto::mock_remote::DisclosureMockRemoteWscdError;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::wscd::DisclosureWscd;
    use itertools::Itertools;
    use p256::ecdsa::Signature;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;
    use p256::elliptic_curve::Generate;
    use rstest::rstest;
    use serde::Deserialize;
    use serde::Serialize;

    use super::ClonePayload;
    use super::CoseError;
    use super::TypedCose;
    use super::sign_coses_without_payload;

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct ToyMessage {
        number: u8,
        string: String,
    }

    #[tokio::test]
    async fn remove_add_payload() {
        let key = SigningKey::generate();
        let payload = ToyMessage {
            number: 42,
            string: "Hello, world!".to_owned(),
        };

        let cose = TypedCose::sign(&payload, Header::default(), &key).await.unwrap();
        let payload_bytes = cose.as_ref().payload.clone().unwrap();

        let without_payload = cose.clone_without_payload();
        assert!(without_payload.as_ref().payload.is_none());

        let with_payload = without_payload.clone_with_payload(payload_bytes);
        let public_key = PublicKey::from(*key.verifying_key());
        assert_eq!(with_payload.verify_and_parse(&public_key).unwrap(), payload);
    }

    /// A WSCD that returns a fixed number of signatures, regardless of how many were requested. Signatures are dropped
    /// or repeated as necessary.
    struct FixedSignatureCountWscd {
        wscd: DisclosureMockRemoteWscd,
        signature_count: usize,
    }

    impl DisclosureWscd for FixedSignatureCountWscd {
        type Key = MockRemoteEcdsaKey;
        type Error = DisclosureMockRemoteWscdError;

        fn new_key<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key {
            self.wscd.new_key(identifier, public_key)
        }

        async fn sign(
            &self,
            messages_and_keys: Vec<(Vec<u8>, Vec<&Self::Key>)>,
        ) -> Result<Vec<Vec<Signature>>, Self::Error> {
            let signatures = self.wscd.sign(messages_and_keys).await?;

            Ok(signatures.into_iter().cycle().take(self.signature_count).collect())
        }
    }

    #[rstest]
    #[case::too_few(1)]
    #[case::too_many(3)]
    #[tokio::test]
    async fn test_sign_coses_error_incorrect_signature_count(#[case] signature_count: usize) {
        let wscd = FixedSignatureCountWscd {
            wscd: DisclosureMockRemoteWscd::default(),
            signature_count,
        };
        let keys_and_challenges = [wscd.wscd.create_random_key(), wscd.wscd.create_random_key()]
            .into_iter()
            .map(|key| (key, b"challenge".as_slice()))
            .collect_vec();

        let error = sign_coses_without_payload(keys_and_challenges, &wscd, Header::default())
            .await
            .expect_err("signing COSEs should not succeed");

        assert_matches!(
            error,
            CoseError::IncorrectSignatureCount { expected: 2, received } if received == signature_count
        );
    }
}
