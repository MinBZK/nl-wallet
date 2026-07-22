//! mdoc-specific COSE helpers.

use std::result::Result;

pub use ::cose::COSE_X5CHAIN_HEADER_LABEL;
pub use ::cose::Cose;
pub use ::cose::CoseError;
pub use ::cose::CoseKey;
pub use ::cose::CoseKeyConversionError;
pub use ::cose::TypedCose;
pub use ::cose::header_with_x5chain;
pub use ::cose::sign_cose;
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
use crypto::wscd::WscdPoa;
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

pub async fn sign_coses<K: CredentialEcdsaKey, P: WscdPoa>(
    keys_and_challenges: Vec<(K, &[u8])>,
    wscd: &impl DisclosureWscd<Key = K, Poa = P>,
    unprotected_header: Header,
    poa_input: P::Input,
    include_payload: bool,
) -> Result<(Vec<CoseSign1>, Option<P>), CoseError> {
    let (keys, challenges): (Vec<_>, Vec<_>) = keys_and_challenges.into_iter().unzip();
    let (signatures_data, protected_header) = signatures_data_and_header(&challenges);

    let keys_and_signature_data = keys
        .iter()
        .zip(signatures_data)
        .map(|(key, signature_data)| (signature_data, vec![key]))
        .collect::<Vec<_>>();

    let result = wscd
        .sign(keys_and_signature_data, poa_input)
        .await
        .map_err(|error| CoseError::Signing(error.into()))?;

    let signed = result
        .signatures
        .into_iter()
        .zip(challenges)
        .map(|(signature, payload)| {
            Ok(CoseSign1 {
                signature: signature.first().ok_or(CoseError::SignatureMissing)?.to_vec(),
                payload: include_payload.then(|| payload.to_vec()),
                protected: protected_header.clone(),
                unprotected: unprotected_header.clone(),
            })
        })
        .collect::<Result<Vec<_>, CoseError>>()?;

    Ok((signed, result.poa))
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
    use coset::Header;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use serde::Deserialize;
    use serde::Serialize;

    use super::ClonePayload;
    use super::TypedCose;

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct ToyMessage {
        number: u8,
        string: String,
    }

    #[tokio::test]
    async fn remove_add_payload() {
        let key = SigningKey::random(&mut OsRng);
        let payload = ToyMessage {
            number: 42,
            string: "Hello, world!".to_owned(),
        };

        let cose = TypedCose::sign(&payload, Header::default(), &key, true).await.unwrap();
        let payload_bytes = cose.as_ref().payload.clone().unwrap();

        let without_payload = cose.clone_without_payload();
        assert!(without_payload.as_ref().payload.is_none());

        let with_payload = without_payload.clone_with_payload(payload_bytes);
        assert_eq!(with_payload.verify_and_parse(key.verifying_key()).unwrap(), payload);
    }
}
