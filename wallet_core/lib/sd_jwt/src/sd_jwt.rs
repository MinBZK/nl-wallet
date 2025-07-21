// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;
use std::fmt::Display;
use std::iter::Peekable;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use derive_more::AsRef;
use derive_more::From;
use indexmap::IndexMap;
use itertools::Itertools;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use jsonwebtoken::Validation;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use ssri::Integrity;

use crypto::EcdsaKeySend;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateUsage;
use jwt::EcdsaDecodingKey;
use jwt::Jwt;
use jwt::VerifiedJwt;
use jwt::jwk::jwk_to_p256;
use utils::generator::Generator;
use utils::spec::SpecOptional;

use crate::decoder::SdObjectDecoder;
use crate::disclosure::Disclosure;
use crate::disclosure::DisclosureContent;
use crate::encoder::ARRAY_DIGEST_KEY;
use crate::encoder::DIGESTS_KEY;
use crate::error::Error;
use crate::error::Result;
use crate::hasher::Hasher;
use crate::hasher::SHA_ALG_NAME;
use crate::key_binding_jwt_claims::KeyBindingJwt;
use crate::key_binding_jwt_claims::KeyBindingJwtBuilder;
use crate::key_binding_jwt_claims::RequiredKeyBinding;

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct SdJwtClaims {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub _sd: Vec<String>,

    pub _sd_alg: Option<String>,

    // Even though we want this to be mandatory, we allow it to be optional in order for the examples from the spec
    // to parse.
    pub cnf: Option<RequiredKeyBinding>,

    // Even though we want this to be mandatory, we allow it to be optional in order for the examples from the spec
    // to parse.
    #[serde(rename = "vct#integrity")]
    pub vct_integrity: Option<Integrity>,

    /// Non-selectively disclosable claims of the SD-JWT.
    #[serde(flatten)]
    pub properties: serde_json::Map<String, serde_json::Value>,
}

/// Representation of an SD-JWT of the format
/// `<Issuer-signed JWT>~<Disclosure 1>~<Disclosure 2>~...~<Disclosure N>~<optional KB-JWT>`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SdJwt {
    issuer_signed_jwt: VerifiedJwt<SdJwtClaims>,

    // To not having to parse the certificates from the JWT header x5c field every time,
    // the certificates are stored here redunantly for convenience as well.
    issuer_certificates: Vec<BorrowingCertificate>,

    disclosures: IndexMap<String, Disclosure>,
}

#[derive(Debug, Clone, Eq, PartialEq, From, AsRef)]
pub struct VerifiedSdJwt(SdJwt);

impl VerifiedSdJwt {
    pub fn into_inner(self) -> SdJwt {
        self.0
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SdJwtPresentation {
    sd_jwt: SdJwt,
    key_binding_jwt: SpecOptional<KeyBindingJwt>,
}

impl SdJwtPresentation {
    /// Parses an SD-JWT into its components as [`SdJwt`].
    ///
    /// ## Error
    /// Returns [`Error::Deserialization`] if parsing fails.
    pub fn parse_and_verify(
        sd_jwt: &str,
        issuer_pubkey: &EcdsaDecodingKey,
        hasher: &impl Hasher,
        kb_expected_aud: &str,
        kb_expected_nonce: &str,
        kb_iat_acceptance_window: Duration,
    ) -> Result<Self> {
        let (rest, kb_segment) = sd_jwt
            .rsplit_once("~")
            .map(|(head, tail)| {
                let head_with_tilde = &sd_jwt[..head.len() + 1];
                (head_with_tilde, tail)
            })
            .ok_or(Error::Deserialization(
                "SD-JWT format is invalid, no segments found".to_string(),
            ))?;

        let sd_jwt = SdJwt::parse_and_verify(rest, issuer_pubkey, hasher)?;

        let Some(RequiredKeyBinding::Jwk(jwk)) = sd_jwt.required_key_bind() else {
            return Err(Error::MissingJwkKeybinding);
        };

        let key_binding_jwt = KeyBindingJwt::parse_and_verify(
            kb_segment,
            &EcdsaDecodingKey::from(&jwk_to_p256(jwk)?),
            kb_expected_aud,
            kb_expected_nonce,
            kb_iat_acceptance_window,
        )?;

        Ok(Self {
            sd_jwt,
            key_binding_jwt: key_binding_jwt.into(),
        })
    }

    pub fn presentation(&self) -> String {
        let disclosures = self.sd_jwt.disclosures.values().map(ToString::to_string).join("~");
        let key_bindings = self.key_binding_jwt.as_ref().to_string();
        [self.sd_jwt.issuer_signed_jwt.jwt().clone().0, disclosures, key_bindings]
            .iter()
            .filter(|segment| !segment.is_empty())
            .join("~")
    }

    pub fn sd_jwt(&self) -> &SdJwt {
        &self.sd_jwt
    }

    pub fn key_binding_jwt(&self) -> &KeyBindingJwt {
        self.key_binding_jwt.as_ref()
    }
}

impl Display for SdJwtPresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&(self.presentation()))
    }
}

impl SdJwt {
    /// Creates a new [`SdJwt`] from its components.
    pub(crate) fn new(
        issuer_signed_jwt: VerifiedJwt<SdJwtClaims>,
        issuer_certificates: Vec<BorrowingCertificate>,
        disclosures: IndexMap<String, Disclosure>,
    ) -> Self {
        Self {
            issuer_signed_jwt,
            issuer_certificates,
            disclosures,
        }
    }

    pub fn header(&self) -> &Header {
        self.issuer_signed_jwt.header()
    }

    pub fn claims(&self) -> &SdJwtClaims {
        self.issuer_signed_jwt.payload()
    }

    pub fn disclosures(&self) -> &IndexMap<String, Disclosure> {
        &self.disclosures
    }

    pub fn required_key_bind(&self) -> Option<&RequiredKeyBinding> {
        self.claims().cnf.as_ref()
    }

    pub fn issuer_certificate_chain(&self) -> &Vec<BorrowingCertificate> {
        &self.issuer_certificates
    }

    pub fn issuer_certificate(&self) -> Option<&BorrowingCertificate> {
        // From https://datatracker.ietf.org/doc/html/rfc7515:
        // The certificate containing the public key corresponding to the key used to digitally sign the
        // JWS MUST be the first certificate.
        self.issuer_certificates.first()
    }

    /// Serializes the components into the final SD-JWT.
    pub fn presentation(&self) -> String {
        let disclosures = self.disclosures.values().join("~");
        if disclosures.is_empty() {
            format!("{}~", self.issuer_signed_jwt.jwt().clone().0)
        } else {
            format!("{}~{}~", self.issuer_signed_jwt.jwt().clone().0, disclosures)
        }
    }

    /// Parses an SD-JWT into its components as [`SdJwt`].
    ///
    /// ## Error
    /// Returns [`Error::Deserialization`] if parsing fails.
    pub fn parse_and_verify(sd_jwt: &str, pubkey: &EcdsaDecodingKey, hasher: &impl Hasher) -> Result<Self> {
        let (jwt, disclosures) = Self::parse_sd_jwt_unverified(sd_jwt, hasher)?;

        let issuer_certificates = jwt.extract_x5c_certificates()?.into();
        let issuer_signed_jwt = VerifiedJwt::try_new(jwt, pubkey, &sd_jwt_validation())?;

        Ok(Self {
            issuer_signed_jwt,
            issuer_certificates,
            disclosures,
        })
    }

    fn parse_sd_jwt_unverified(
        sd_jwt: &str,
        hasher: &impl Hasher,
    ) -> Result<(Jwt<SdJwtClaims>, IndexMap<String, Disclosure>)> {
        if !sd_jwt.ends_with("~") {
            return Err(Error::Deserialization(
                "SD-JWT format is invalid, input doesn't and with '~'".to_string(),
            ));
        }

        let (sd_jwt_segment, disclosure_segments) = sd_jwt.split_once('~').ok_or(Error::Deserialization(
            "SD-JWT format is invalid, input doesn't contain a '~'".to_string(),
        ))?;

        let jwt: Jwt<SdJwtClaims> = sd_jwt_segment.parse()?;

        let disclosures = disclosure_segments
            .split("~")
            .filter(|segment| !segment.is_empty())
            .try_fold(IndexMap::new(), |mut acc, segment| {
                let disclosure = Disclosure::parse(segment)?;
                acc.insert(hasher.encoded_digest(disclosure.as_str()), disclosure);
                Ok::<_, Error>(acc)
            })?;

        Ok((jwt, disclosures))
    }

    /// Prepares this [`SdJwt`] for a presentation, returning an [`SdJwtPresentationBuilder`].
    /// ## Errors
    /// - [`Error::InvalidHasher`] is returned if the provided `hasher`'s algorithm doesn't match the algorithm
    ///   specified by SD-JWT's `_sd_alg` claim. "sha-256" is used if the claim is missing.
    pub fn into_presentation_builder(
        self,
        hasher: &dyn Hasher,
        key_binding_iat: DateTime<Utc>,
        key_binding_aud: String,
        key_binding_nonce: String,
        key_binding_alg: Algorithm,
    ) -> Result<SdJwtPresentationBuilder> {
        SdJwtPresentationBuilder::new(
            self,
            KeyBindingJwtBuilder::new(key_binding_iat, key_binding_aud, key_binding_nonce, key_binding_alg),
            hasher,
        )
    }

    /// Returns the JSON object obtained by replacing all disclosures into their
    /// corresponding JWT concealable claims.
    pub fn into_disclosed_object(self) -> Result<serde_json::Map<String, serde_json::Value>> {
        let decoder = SdObjectDecoder;
        let object = serde_json::to_value(self.claims())?;

        decoder.decode(object.as_object().unwrap(), &self.disclosures)
    }
}

impl Display for SdJwt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&(self.presentation()))
    }
}

impl VerifiedSdJwt {
    /// Parses an SD-JWT into its components as [`VerifiedSdJwt`] verifying against the provided trust anchors.
    pub fn parse_and_verify_against_trust_anchors(
        sd_jwt: &str,
        hasher: &impl Hasher,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<VerifiedSdJwt> {
        let (jwt, disclosures) = SdJwt::parse_sd_jwt_unverified(sd_jwt, hasher)?;

        let (issuer_signed_jwt, issuer_certificate) = VerifiedJwt::try_new_against_trust_anchors(
            jwt,
            &sd_jwt_validation(),
            time,
            CertificateUsage::Mdl,
            trust_anchors,
        )?;

        Ok(Self(SdJwt {
            issuer_signed_jwt,
            issuer_certificates: vec![issuer_certificate],
            disclosures,
        }))
    }

    /// Parses an SD-JWT into its components as [`VerifiedSdJwt`] without verifying the signature.
    ///
    /// ## Error
    /// Returns [`Error::Deserialization`] if parsing fails.
    pub fn dangerous_parse_unverified(sd_jwt: &str, hasher: &impl Hasher) -> Result<Self> {
        let (jwt, disclosures) = SdJwt::parse_sd_jwt_unverified(sd_jwt, hasher)?;

        let issuer_certificates = jwt.extract_x5c_certificates()?.into();
        let issuer_signed_jwt = VerifiedJwt::new_dangerous(jwt)?;

        Ok(Self(SdJwt {
            issuer_signed_jwt,
            issuer_certificates,
            disclosures,
        }))
    }
}

#[derive(Clone)]
pub struct SdJwtPresentationBuilder<'a> {
    sd_jwt: SdJwt,
    kb_jwt_builder: KeyBindingJwtBuilder,
    hasher: &'a dyn Hasher,

    /// Non-disclosed attributes. All attributes start here. Calling `disclose()` moves an attribute from here
    /// to `disclosed`.
    nondisclosed: IndexMap<String, Disclosure>,

    /// Attributes to be disclosed.
    disclosures: IndexMap<String, Disclosure>,

    /// A helper object containing both non-selectively disclosable JWT claims and the `_sd` hashes,
    /// used by `digests_to_disclose()`.
    full_payload: serde_json::Value,
}

impl<'a> SdJwtPresentationBuilder<'a> {
    pub(crate) fn new(sd_jwt: SdJwt, kb_jwt_builder: KeyBindingJwtBuilder, hasher: &'a dyn Hasher) -> Result<Self> {
        let required_hasher = sd_jwt.claims()._sd_alg.as_deref().unwrap_or(SHA_ALG_NAME);
        if required_hasher != hasher.alg_name() {
            return Err(Error::InvalidHasher(format!(
                "hasher \"{}\" was provided, but \"{required_hasher} is required\"",
                hasher.alg_name()
            )));
        }

        let full_payload = {
            let claims = sd_jwt.issuer_signed_jwt.payload().clone();
            let sd = claims._sd.into_iter().map(serde_json::Value::String).collect();

            let mut payload = claims.properties;
            payload.insert(DIGESTS_KEY.to_string(), serde_json::Value::Array(sd));

            serde_json::Value::Object(payload)
        };

        let nondisclosed = sd_jwt.disclosures.clone();

        Ok(Self {
            sd_jwt,
            kb_jwt_builder,
            nondisclosed,
            disclosures: IndexMap::new(),
            full_payload,
            hasher,
        })
    }

    pub fn disclose(mut self, path: &str) -> Result<Self> {
        let path_segments = path.trim_start_matches('/').split('/').peekable();
        let digests = digests_to_disclose(&self.full_payload, path_segments, &self.nondisclosed)?
            .into_iter()
            // needed, since some strings are borrowed for the lifetime of the borrow of `self.disclosures`.
            .map(ToOwned::to_owned)
            // needed, to drop borrow `self.disclosures`.
            .collect_vec();

        digests.into_iter().for_each(|digest| {
            if let Some(disclosure) = self.nondisclosed.shift_remove(&digest) {
                self.disclosures.insert(digest, disclosure);
            }
        });

        Ok(self)
    }

    /// Returns the resulting [`SdJwtPresentation`].
    pub async fn finish(self, signing_key: &impl EcdsaKeySend) -> Result<SdJwtPresentation> {
        // Put everything back in its place.
        let SdJwtPresentationBuilder {
            mut sd_jwt,
            disclosures,
            kb_jwt_builder,
            ..
        } = self;
        sd_jwt.disclosures = disclosures;

        let key_binding_jwt = kb_jwt_builder.finish(&sd_jwt, self.hasher, signing_key).await?;

        let sd_jwt_presentation = SdJwtPresentation {
            sd_jwt,
            key_binding_jwt: key_binding_jwt.into(),
        };

        Ok(sd_jwt_presentation)
    }
}

pub(crate) fn sd_jwt_validation() -> Validation {
    let mut validation = Validation::new(Algorithm::ES256);
    validation.validate_aud = false;
    validation.validate_nbf = true;
    validation.leeway = 0;
    validation.required_spec_claims = HashSet::new();
    validation
}

/// Recursively searches for the specified path in the object and disclosures, returning the digests
/// of objects which are to be disclosed in order to disclose the specified `path.`
///
/// The `object` must be the payload of an SD-JWT, containing an `_sd` array and other claims.
fn digests_to_disclose<'a, I>(
    object: &'a serde_json::Value,
    mut path: Peekable<I>,
    disclosures: &'a IndexMap<String, Disclosure>,
) -> Result<Vec<&'a str>>
where
    I: Iterator<Item = &'a str>,
{
    let element_key = path
        .next()
        .ok_or_else(|| Error::InvalidPath("element at path doesn't exist or is not disclosable".to_string()))?;
    let has_next = path.peek().is_some();
    match object {
        // We are just traversing to a deeper part of the object.
        serde_json::Value::Object(object) if has_next => {
            let next_object = object
                .get(element_key)
                .or_else(|| {
                    find_disclosure_digest(object, element_key, disclosures)
                        .and_then(|digest| disclosures.get(digest))
                        .map(|disclosure| disclosure.claim_value())
                })
                .ok_or_else(|| {
                    Error::InvalidPath("the referenced element doesn't exist or is not concealable".to_string())
                })?;

            digests_to_disclose(next_object, path, disclosures)
        }
        // We reached the parent of the value we want to disclose.
        // Make sure it's disclosable by finding the digest of its disclosure.
        serde_json::Value::Object(object) => {
            let digest = find_disclosure_digest(object, element_key, disclosures).ok_or_else(|| {
                Error::InvalidPath("the referenced element doesn't exist or is not concealable".to_string())
            })?;
            let disclosure = disclosures.get(digest).unwrap();
            let mut sub_disclosures: Vec<&str> =
                get_all_sub_disclosures(disclosure.claim_value(), disclosures).collect();
            sub_disclosures.push(digest);
            Ok(sub_disclosures)
        }
        // Traversing an array
        serde_json::Value::Array(arr) if has_next => {
            let index = element_key
                .parse::<usize>()
                .ok()
                .filter(|idx| arr.len() > *idx)
                .ok_or_else(|| Error::InvalidPath(String::default()))?;
            let next_object = arr.get(index).ok_or_else(|| {
                Error::InvalidPath("the referenced element doesn't exist or is not concealable".to_string())
            })?;

            digests_to_disclose(next_object, path, disclosures)
        }
        // Concealing an array's entry.
        serde_json::Value::Array(arr) => {
            let index = element_key
                .parse::<usize>()
                .ok()
                .filter(|idx| arr.len() > *idx)
                .ok_or_else(|| Error::InvalidPath(String::default()))?;
            let digest = arr
                .get(index)
                .unwrap()
                .as_object()
                .and_then(|entry| find_disclosure_digest(entry, "", disclosures))
                .ok_or_else(|| {
                    Error::InvalidPath("the referenced element doesn't exist or is not concealable".to_string())
                })?;
            let disclosure = disclosures.get(digest).unwrap();
            let mut sub_disclosures: Vec<&str> =
                get_all_sub_disclosures(disclosure.claim_value(), disclosures).collect();
            sub_disclosures.push(digest);
            Ok(sub_disclosures)
        }
        _ => Err(Error::InvalidPath(String::default())),
    }
}

/// Find the digest of the given `key` in the `object` and `disclosures`.
fn find_disclosure_digest<'o>(
    object: &'o serde_json::Map<String, serde_json::Value>,
    key: &str,
    disclosures: &IndexMap<String, Disclosure>,
) -> Option<&'o str> {
    let maybe_disclosable_array_entry = || {
        object
            .get(ARRAY_DIGEST_KEY)
            .and_then(|value| value.as_str())
            .filter(|_| object.len() == 1)
    };
    // Try to find the digest for disclosable property `key` in
    // the `_sd` field of `object`.
    object
        .get(DIGESTS_KEY)
        .and_then(|value| value.as_array())
        .iter()
        .flat_map(|values| values.iter())
        .flat_map(|value| value.as_str())
        .find(|digest| {
            disclosures
                .get(*digest)
                .and_then(|disclosure| match &disclosure.content {
                    DisclosureContent::ObjectProperty(_, name, _) => Some(name),
                    _ => None,
                })
                .is_some_and(|name| name == key)
        })
        // If no result is found try checking `object` as a disclosable array entry.
        .or_else(maybe_disclosable_array_entry)
}

fn get_all_sub_disclosures<'a>(
    start: &'a serde_json::Value,
    disclosures: &'a IndexMap<String, Disclosure>,
) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    match start {
        // `start` is a JSON object, check if it has a "_sd" array + recursively
        // check all its properties
        serde_json::Value::Object(object) => {
            let direct_sds = object
                .get(DIGESTS_KEY)
                .and_then(|sd| sd.as_array())
                .map(|sd| sd.iter())
                .unwrap_or_default()
                .flat_map(|value| value.as_str())
                .filter(|digest| disclosures.contains_key(*digest));
            let sub_sds = object
                .values()
                .flat_map(|value| get_all_sub_disclosures(value, disclosures));
            Box::new(itertools::chain!(direct_sds, sub_sds))
        }
        // `start` is a JSON array, check for disclosable values `{"...", <digest>}` +
        // recursively check all its values.
        serde_json::Value::Array(arr) => {
            let mut digests = vec![];
            for value in arr {
                if let Some(serde_json::Value::String(digest)) = value.get(ARRAY_DIGEST_KEY) {
                    if disclosures.contains_key(digest) {
                        digests.push(digest.as_str());
                    }
                } else {
                    get_all_sub_disclosures(value, disclosures).for_each(|digest| digests.push(digest));
                }
            }
            Box::new(digests.into_iter())
        }
        _ => Box::new(std::iter::empty()),
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use assert_matches::assert_matches;
    use chrono::Duration;
    use chrono::Utc;
    use futures::FutureExt;
    use itertools::Itertools;
    use jsonwebtoken::Algorithm;
    use jsonwebtoken::errors::ErrorKind;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use rstest::rstest;
    use serde_json::json;
    use ssri::Integrity;

    use jwt::EcdsaDecodingKey;
    use jwt::error::JwtError;

    use crate::builder::SdJwtBuilder;
    use crate::disclosure::DisclosureContent;
    use crate::examples::*;
    use crate::hasher::Sha256Hasher;
    use crate::sd_jwt::Error;
    use crate::sd_jwt::SdJwt;
    use crate::sd_jwt::SdJwtPresentation;

    #[rstest]
    #[case(SIMPLE_STRUCTURED_SD_JWT)]
    #[case(COMPLEX_STRUCTURED_SD_JWT)]
    #[case(SD_JWT_VC)]
    fn parse_various(#[case] encoded_sd_jwt: &str) {
        SdJwt::parse_and_verify(encoded_sd_jwt, &examples_sd_jwt_decoding_key(), &Sha256Hasher).unwrap();
    }

    #[test]
    fn parse_kb() {
        SdJwtPresentation::parse_and_verify(
            WITH_KB_SD_JWT,
            &examples_sd_jwt_decoding_key(),
            &Sha256Hasher,
            WITH_KB_SD_JWT_AUD,
            WITH_KB_SD_JWT_NONCE,
            Duration::days(36500),
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_parse_should_error_for_expired_jwt() {
        let signing_key = SigningKey::random(&mut OsRng);
        let holder_privkey = SigningKey::random(&mut OsRng);

        let sd_jwt = SdJwtBuilder::new(json!({
            "exp": (Utc::now() - Duration::days(1)).timestamp(),
        }))
        .unwrap()
        .finish(
            Algorithm::ES256,
            Integrity::from(""),
            &signing_key,
            vec![],
            holder_privkey.verifying_key(),
        )
        .await
        .unwrap()
        .to_string();

        let err = SdJwt::parse_and_verify(
            &sd_jwt,
            &EcdsaDecodingKey::from(signing_key.verifying_key()),
            &Sha256Hasher,
        )
        .expect_err("should fail");

        assert_matches!(err, Error::JwtParsing(JwtError::Validation(err)) if err.kind() == &ErrorKind::ExpiredSignature);
    }

    #[test]
    fn parse() {
        let sd_jwt =
            SdJwt::parse_and_verify(SIMPLE_STRUCTURED_SD_JWT, &examples_sd_jwt_decoding_key(), &Sha256Hasher).unwrap();
        assert_eq!(sd_jwt.disclosures.len(), 2);
    }

    #[test]
    fn parse_vc() {
        let sd_jwt = SdJwt::parse_and_verify(SD_JWT_VC, &examples_sd_jwt_decoding_key(), &Sha256Hasher).unwrap();
        assert_eq!(sd_jwt.disclosures.len(), 21);
        assert!(sd_jwt.required_key_bind().is_some());
    }

    #[test]
    fn round_trip_ser_des() {
        let sd_jwt =
            SdJwt::parse_and_verify(SIMPLE_STRUCTURED_SD_JWT, &examples_sd_jwt_decoding_key(), &Sha256Hasher).unwrap();
        assert_eq!(&sd_jwt.to_string(), SIMPLE_STRUCTURED_SD_JWT);
    }

    fn create_presentation(
        object: serde_json::Value,
        conceal_paths: &[&str],
        disclose_paths: &[&str],
    ) -> SdJwtPresentation {
        let issuer_privkey = SigningKey::random(&mut OsRng);
        let holder_privkey = SigningKey::random(&mut OsRng);

        let sd_jwt = conceal_paths
            .iter()
            .fold(SdJwtBuilder::new(object).unwrap(), |builder, path| {
                builder.make_concealable(path).unwrap()
            })
            .finish(
                Algorithm::ES256,
                Integrity::from(""),
                &issuer_privkey,
                vec![],
                holder_privkey.verifying_key(),
            )
            .now_or_never()
            .unwrap()
            .unwrap();

        disclose_paths
            .iter()
            .fold(
                sd_jwt
                    .into_presentation_builder(
                        &Sha256Hasher,
                        Utc::now(),
                        "aud".to_string(),
                        "nonce".to_string(),
                        Algorithm::ES256,
                    )
                    .unwrap(),
                |builder, path| builder.disclose(path).unwrap(),
            )
            .finish(&holder_privkey)
            .now_or_never()
            .unwrap()
            .unwrap()
    }

    #[rstest]
    #[case(
        json!({"given_name": "John", "family_name": "Doe"}),
        &["/given_name", "/family_name"],
        &[],
        &[],
        &[],
    )]
    #[case(
        json!({"given_name": "John", "family_name": "Doe"}),
        &["/given_name", "/family_name"],
        &["/given_name"],
        &["given_name"],
        &[],
    )]
    #[case(
        json!({"given_name": "John", "family_name": "Doe"}),
        &["/given_name", "/family_name"],
        &["/given_name", "/family_name"],
        &["given_name", "family_name"],
        &[],
    )]
    #[case(
        json!({"given_name": "John", "family_name": "Doe"}),
        &["/given_name"],
        &["/given_name"],
        &["given_name"],
        &["/family_name"],
    )]
    #[case(
        json!({"given_name": "John", "family_name": "Doe"}),
        &[],
        &[],
        &[],
        &["/family_name", "/given_name"],
    )]
    #[case(
        json!({"address": {"street": "Main st.", "house_number": 4 }}),
        &["/address/street"],
        &["/address/street"],
        &["street"],
        &["/address", "/address/house_number"],
    )]
    #[case(
        json!({"address": {"street": "Main st.", "house_number": 4 }}),
        &["/address/street", "/address"],
        &["/address/street"],
        &["street"],
        &[],
    )]
    #[case(
        json!({"address": {"street": "Main st.", "house_number": 4 }}),
        &["/address/street", "/address/house_number", "/address"],
        &["/address/street", "/address/house_number", "/address"],
        &["street", "house_number", "address"],
        &[],
    )]
    #[case(
        json!({"address": {"street": "Main st.", "house_number": 4 }}),
        &["/address"],
        &["/address"],
        &["address"],
        &[],
    )]
    #[case(
        json!({"nationalities": ["NL", "DE"]}),
        &["/nationalities"],
        &["/nationalities"],
        &["nationalities"],
        &[],
    )]
    fn test_selectively_disclosable_attributes_in_presentation(
        #[case] object: serde_json::Value,
        #[case] conceal_paths: &[&str],
        #[case] disclose_paths: &[&str],
        #[case] expected_disclosed_paths: &[&str],
        #[case] expected_not_selectively_disclosable_paths: &[&str],
    ) {
        let presentation = create_presentation(object, conceal_paths, disclose_paths);

        fn get_paths(object: &serde_json::Map<String, serde_json::Value>) -> HashSet<String> {
            fn traverse(value: &serde_json::Value, current_path: &str, paths: &mut HashSet<String>) {
                if let serde_json::Value::Object(map) = value {
                    for (key, val) in map {
                        let new_path = if current_path.is_empty() {
                            format!("/{key}")
                        } else {
                            format!("{current_path}/{key}")
                        };

                        if key != "_sd" {
                            paths.insert(new_path.clone());
                            if let serde_json::Value::Object(_) = val {
                                traverse(val, &new_path, paths)
                            }
                        }
                    }
                }
            }

            let mut paths = HashSet::new();
            traverse(&serde_json::Value::Object(object.clone()), "", &mut paths);
            paths
        }

        let claims = presentation.sd_jwt.issuer_signed_jwt.payload();
        let not_selectively_disclosable_paths = get_paths(&claims.properties);

        assert_eq!(
            expected_disclosed_paths,
            &presentation
                .sd_jwt
                .disclosures
                .into_iter()
                .filter_map(|(_, disclosure)| match disclosure.content {
                    DisclosureContent::ObjectProperty(_, name, _) => Some(name),
                    _ => None,
                })
                .collect_vec()
        );

        assert_eq!(
            expected_not_selectively_disclosable_paths
                .iter()
                .map(|path| String::from(*path))
                .collect::<HashSet<_>>(),
            not_selectively_disclosable_paths
        );
    }
}
