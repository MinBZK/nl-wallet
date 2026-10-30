use std::collections::HashSet;
use std::collections::VecDeque;

use error_category::ErrorCategory;
use itertools::Either;
use itertools::Itertools;
use jwe::algorithm::CompressionAlgorithm;
use jwe::algorithm::EncryptionAlgorithm;
use jwe::encryption::JwePublicKey;
use jwe::error::EcdhPublicJwkError;
use nonempty_collections::NESet;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DisplayFromStr;
use serde_with::serde_as;
use utils::vec_at_least::VecNonEmpty;

use crate::jwe::JweCompressionAlgorithm;
use crate::jwe::JweEncryptionAlgorithm;
use crate::metadata::issuer_metadata::CredentialRequestEncryption;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum CredentialRequestEncryptionMetadataError {
    #[error("missing \"kid\" value of JWK(s) at index: {}", .0.iter().join(", "))]
    #[category(critical)]
    MissingKid(VecNonEmpty<usize>),

    #[error("\"kid\" value(s) used by multiple JWKs: {}", .0.iter().join(", "))]
    #[category(pd)]
    DuplicateKid(NESet<String>),

    #[error(
        "none of the JWKs contained a supported algorithm or were internally consistent: {}",
        std::iter::once(first).chain(subsequent).join(", ")
    )]
    #[category(pd)]
    NoSupportedJwk {
        #[source]
        first: EcdhPublicJwkError,
        subsequent: Vec<EcdhPublicJwkError>,
    },

    #[error("no supported encryption algorithm found among those listed: {}", .0.iter().join(", "))]
    #[category(pd)]
    NoSupportedEncryptionAlgorithm(NESet<JweEncryptionAlgorithm>),
}

/// Data type that is the result of the selecting a Credential Request encryption JWK to use amongst those present in
/// the Issuer Metadata. The Wallet uses the values within this type to encrypt Credential Requests before sending them
/// to the Issuer.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CredentialRequestEncryptionParameters {
    pub public_key: JwePublicKey,

    #[serde_as(as = "DisplayFromStr")]
    pub encryption_algorithm: EncryptionAlgorithm,

    #[serde_as(as = "Option<DisplayFromStr>")]
    pub compression: Option<CompressionAlgorithm>,
}

impl CredentialRequestEncryptionParameters {
    pub fn new(
        public_key: JwePublicKey,
        encryption_algorithm: EncryptionAlgorithm,
        compression: Option<CompressionAlgorithm>,
    ) -> Self {
        Self {
            public_key,
            encryption_algorithm,
            compression,
        }
    }
}

impl CredentialRequestEncryption {
    /// Selection algorithm for the JWKs present in the `credential_request_encryption` section of the Issuer Metadata.
    /// If the metadata indicates that request encryption is mandatory through the `encryption_required` field, it
    /// constitutes an error of none of the JWKs contain a valid and supported public key. If encryption is optional,
    /// this method will return `None`.
    pub(crate) fn select_encryption_parameters(
        &self,
    ) -> Result<Option<CredentialRequestEncryptionParameters>, CredentialRequestEncryptionMetadataError> {
        // According to the OpenID4VCI spec, every advertised Credential Request encryption JWK must have a `kid` value.
        let (kids, missing_kid_indices): (Vec<_>, Vec<_>) =
            self.jwks
                .keys
                .iter()
                .enumerate()
                .partition_map(|(index, jwk)| match jwk.kid() {
                    Some(kid) => Either::Left(kid),
                    None => Either::Right(index),
                });

        if let Ok(missing_kid_indices) = VecNonEmpty::try_from(missing_kid_indices) {
            return Err(CredentialRequestEncryptionMetadataError::MissingKid(
                missing_kid_indices,
            ));
        }

        // According to the OpenID4VCI spec, the `kid` value should be unique to each JWK.
        let duplicate_kids = kids
            .into_iter()
            .duplicates()
            .map(str::to_string)
            .collect::<HashSet<_>>();

        if let Some(duplicate_kids) = NESet::try_from_set(duplicate_kids) {
            return Err(CredentialRequestEncryptionMetadataError::DuplicateKid(duplicate_kids));
        }

        // Find the first JWK that contains a valid and supported public key, while collecting the error for each
        // unsupported JWK.
        let mut jwk_errors = VecDeque::new();
        let public_key = self
            .jwks
            .keys
            .iter()
            .find_map(|jwk| match JwePublicKey::try_from_jwk(jwk) {
                Ok(public_key) => Some(public_key),
                Err(error) => {
                    jwk_errors.push_back(error);
                    None
                }
            });

        let Some(public_key) = public_key else {
            if self.encryption_required {
                // If no supported JWK is found and encryption is mandatory, return an error containing all of the JWK
                // errors encountered.
                let first = jwk_errors.pop_front().expect(
                    "jwk_errors should contain at least one value, as self.jwks.keys is non-empty and none of the \
                     JWKs parsed successfully",
                );

                return Err(CredentialRequestEncryptionMetadataError::NoSupportedJwk {
                    first,
                    subsequent: jwk_errors.into(),
                });
            } else {
                // If encryption is mandatory, succeed and return no encryption parameters.
                return Ok(None);
            }
        };

        // Amongst the supported encryption algorithm, pick one based on preference. Return an error if all of them are
        // unsupported.
        let encryption_algorithm = JweEncryptionAlgorithm::find_preferred_known(&self.enc_values_supported)
            .ok_or_else(|| {
                CredentialRequestEncryptionMetadataError::NoSupportedEncryptionAlgorithm(
                    self.enc_values_supported.clone(),
                )
            })?;

        // Find the first compression algorithm that the Issuer supports, which will always be "deflate" as this is the
        // only one known.
        let compression = self
            .zip_values_supported
            .iter()
            .find_map(|compression| match compression {
                JweCompressionAlgorithm::Known(compression) => Some(*compression),
                JweCompressionAlgorithm::Unknown(_) => None,
            });

        let parameters = CredentialRequestEncryptionParameters::new(public_key, encryption_algorithm, compression);

        Ok(Some(parameters))
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashSet;

    use jwe::algorithm::EncryptionAlgorithm;
    use jwk_simple::Algorithm;
    use jwk_simple::EcCurve;
    use jwk_simple::EcParams;
    use jwk_simple::Key;
    use jwk_simple::KeyParams;
    use jwk_simple::KeyUse;
    use jwk_simple::RsaParams;
    use jwk_simple::encoding::Base64UrlBytes;
    use nonempty_collections::NESet;
    use nonempty_collections::nes;
    use p256::SecretKey;
    use p256::elliptic_curve::Generate;
    use p256::elliptic_curve::sec1::ToSec1Point;
    use rstest::rstest;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

    use super::CredentialRequestEncryptionMetadataError;
    use crate::jwe::JweEncryptionAlgorithm;
    use crate::jwks::Jwks;
    use crate::metadata::issuer_metadata::CredentialRequestEncryption;

    fn ec_key(curve: EcCurve, index: Option<usize>) -> Key {
        let encoded_point = SecretKey::generate().public_key().to_sec1_point(false);

        let key = Key::new(KeyParams::Ec(EcParams::new_public(
            curve,
            encoded_point.x().unwrap().to_vec().into(),
            encoded_point.y().unwrap().to_vec().into(),
        )))
        .with_alg(Algorithm::EcdhEs)
        .with_use(KeyUse::Encryption);

        if let Some(index) = index {
            key.with_kid(format!("ec-key-{index}"))
        } else {
            key
        }
    }

    fn valid_ec_key(index: usize) -> Key {
        ec_key(EcCurve::P256, Some(index))
    }

    fn invalid_ec_key(index: usize) -> Key {
        ec_key(EcCurve::P521, Some(index))
    }

    fn ec_key_without_kid() -> Key {
        ec_key(EcCurve::P256, None)
    }

    fn rsa_key(index: usize) -> Key {
        Key::new(KeyParams::Rsa(RsaParams::new_public(
            Base64UrlBytes::new(vec![1, 2, 3]),
            Base64UrlBytes::new(vec![1, 0, 1]),
        )))
        .with_kid(format!("rsa-key-{index}"))
        .with_alg(Algorithm::Rs256)
        .with_use(KeyUse::Encryption)
    }

    fn credential_request_encryption_with_algs(
        keys: VecNonEmpty<Key>,
        enc_values_supported: NESet<JweEncryptionAlgorithm>,
        encryption_required: bool,
    ) -> CredentialRequestEncryption {
        CredentialRequestEncryption {
            jwks: Jwks::new(keys),
            enc_values_supported,
            zip_values_supported: HashSet::new(),
            encryption_required,
        }
    }

    fn credential_request_encryption(keys: VecNonEmpty<Key>, encryption_required: bool) -> CredentialRequestEncryption {
        credential_request_encryption_with_algs(
            keys,
            nes![EncryptionAlgorithm::A128Gcm.into(), EncryptionAlgorithm::A256Gcm.into()],
            encryption_required,
        )
    }

    #[rstest]
    fn test_ok(#[values(false, true)] encryption_required: bool) {
        // Given a list of JWKs, the selection procedure should choose the first one that is supported and validates
        // correctly.
        let parameters = credential_request_encryption(
            vec_nonempty![invalid_ec_key(0), rsa_key(1), valid_ec_key(2), valid_ec_key(3)],
            encryption_required,
        )
        .select_encryption_parameters()
        .expect("selecting parameters from credential request encryption metadata should succeed")
        .expect("a public key should be selected");

        assert_eq!(parameters.public_key.id(), Some("ec-key-2"));
        assert_eq!(parameters.encryption_algorithm, EncryptionAlgorithm::A256Gcm);
        assert_eq!(parameters.compression, None);
    }

    #[rstest]
    fn test_error_missing_kid(#[values(false, true)] encryption_required: bool) {
        // If any of the provided JWKs is missing a `kid` value the selection procedure should return an error, even if
        // one or more of the keys are valid.
        let error = credential_request_encryption(
            vec_nonempty![
                valid_ec_key(0),
                ec_key_without_kid(),
                valid_ec_key(2),
                ec_key_without_kid()
            ],
            encryption_required,
        )
        .select_encryption_parameters()
        .expect_err("selecting parameters from credential request encryption metadata should fail");

        assert_matches!(
            error,
            CredentialRequestEncryptionMetadataError::MissingKid(indices) if indices == vec_nonempty![1, 3]
        );
    }

    #[rstest]
    fn test_error_duplicate_kid(#[values(false, true)] encryption_required: bool) {
        // If any of the `kid` values in the JWKs appear more than once the selection procedure should return an error,
        // even if one or more of the keys are valid.
        let error = credential_request_encryption(
            vec_nonempty![
                valid_ec_key(0),
                valid_ec_key(1),
                valid_ec_key(2),
                valid_ec_key(3),
                valid_ec_key(3),
                valid_ec_key(1),
            ],
            encryption_required,
        )
        .select_encryption_parameters()
        .expect_err("selecting parameters from credential request encryption metadata should fail");

        assert_matches!(
            error,
            CredentialRequestEncryptionMetadataError::DuplicateKid(kids)
                if kids == nes!["ec-key-1".to_string(), "ec-key-3".to_string()]
        );
    }

    #[rstest]
    fn test_error_no_supported_jwk(#[values(false, true)] encryption_required: bool) {
        // If all of the JWKs are invalid or use unsupported algorithms, the selection procedure should return an error
        // only if encryption is mandatory. Otherwise, it should return `None`.
        let result = credential_request_encryption(
            vec_nonempty![
                invalid_ec_key(0),
                invalid_ec_key(1),
                rsa_key(2),
                invalid_ec_key(3),
                rsa_key(4),
            ],
            encryption_required,
        )
        .select_encryption_parameters();

        if encryption_required {
            let error =
                result.expect_err("selecting parameters from credential request encryption metadata should fail");

            assert_matches!(
                error,
                CredentialRequestEncryptionMetadataError::NoSupportedJwk { subsequent, .. } if subsequent.len() == 4
            );
        } else {
            let parameters =
                result.expect("selecting parameters from credential request encryption metadata should succeed");

            assert!(parameters.is_none());
        }
    }

    #[rstest]
    fn test_error_no_supported_encryption_algorithm(#[values(false, true)] encryption_required: bool) {
        // If the metadata only contains unsupported encryption algorithms, the selection procedure should return an
        // error.
        let enc_values_supported = nes![
            JweEncryptionAlgorithm::Unknown("foo".to_string()),
            JweEncryptionAlgorithm::Unknown("bar".to_string())
        ];
        let error = credential_request_encryption_with_algs(
            vec_nonempty![valid_ec_key(0), valid_ec_key(1), valid_ec_key(2)],
            enc_values_supported.clone(),
            encryption_required,
        )
        .select_encryption_parameters()
        .expect_err("selecting parameters from credential request encryption metadata should fail");

        assert_matches!(
            error,
            CredentialRequestEncryptionMetadataError::NoSupportedEncryptionAlgorithm(algs)
                if algs == enc_values_supported
        );
    }
}
