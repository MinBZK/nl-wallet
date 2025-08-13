// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::iter::Peekable;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use derive_more::AsRef;
use derive_more::From;
use itertools::Itertools;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use jsonwebtoken::Validation;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use ssri::Integrity;

use attestation_types::claim_path::ClaimPath;
use crypto::EcdsaKeySend;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateUsage;
use jwt::EcdsaDecodingKey;
use jwt::Jwt;
use jwt::VerifiedJwt;
use jwt::jwk::jwk_to_p256;
use utils::generator::Generator;
use utils::spec::SpecOptional;
use utils::vec_at_least::VecNonEmpty;

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

    disclosures: HashMap<String, Disclosure>,
}

#[derive(Debug, Clone, Eq, PartialEq, From, AsRef)]
pub struct VerifiedSdJwt(SdJwt);

impl VerifiedSdJwt {
    pub fn into_inner(self) -> SdJwt {
        self.0
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnsignedSdJwtPresentation(SdJwt);

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
        let disclosures = self.sd_jwt.disclosures.values().join("~");
        let key_bindings = self.key_binding_jwt.as_ref().to_string();
        [self.sd_jwt.issuer_signed_jwt.jwt().clone().0, disclosures, key_bindings]
            .into_iter()
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
        disclosures: HashMap<String, Disclosure>,
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

    pub fn disclosures(&self) -> &HashMap<String, Disclosure> {
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
    ) -> Result<(Jwt<SdJwtClaims>, HashMap<String, Disclosure>)> {
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
            .try_fold(HashMap::new(), |mut acc, segment| {
                let disclosure = Disclosure::parse(segment)?;
                acc.insert(hasher.encoded_digest(disclosure.as_str()), disclosure);
                Ok::<_, Error>(acc)
            })?;

        Ok((jwt, disclosures))
    }

    /// Prepares this [`SdJwt`] for a presentation, returning an [`SdJwtPresentationBuilder`].
    pub fn into_presentation_builder(self) -> SdJwtPresentationBuilder {
        SdJwtPresentationBuilder::new(self)
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
pub struct SdJwtPresentationBuilder {
    sd_jwt: SdJwt,

    /// Non-disclosed attributes. All attributes start here. Calling `disclose()` moves an attribute from here
    /// to `disclosed`.
    nondisclosed: HashMap<String, Disclosure>,

    /// Digests to be disclosed.
    digests_to_be_disclosed: HashSet<String>,

    /// A helper object containing both non-selectively disclosable JWT claims and the `_sd` hashes,
    /// used by `digests_to_disclose()`.
    full_payload: serde_json::Value,
}

impl SdJwtPresentationBuilder {
    pub(crate) fn new(mut sd_jwt: SdJwt) -> Self {
        let full_payload = {
            let claims = sd_jwt.issuer_signed_jwt.payload().clone();
            let sd = claims._sd.into_iter().map(serde_json::Value::String).collect();

            let mut payload = claims.properties;
            payload.insert(DIGESTS_KEY.to_string(), serde_json::Value::Array(sd));

            serde_json::Value::Object(payload)
        };

        let nondisclosed = std::mem::take(&mut sd_jwt.disclosures);

        Self {
            sd_jwt,
            nondisclosed,
            digests_to_be_disclosed: HashSet::new(),
            full_payload,
        }
    }

    pub fn disclose(mut self, path: &VecNonEmpty<ClaimPath>) -> Result<Self> {
        // Gather all digests to be disclosed into a set. This can include intermediary attributes as well

        self.digests_to_be_disclosed.extend({
            let mut path_segments = path.iter().peekable();
            digests_to_disclose(&self.full_payload, &mut path_segments, &self.nondisclosed, false)?
                .into_iter()
                .map(String::from)
        });

        Ok(self)
    }

    pub fn finish(self) -> UnsignedSdJwtPresentation {
        // Put everything back in its place.
        let SdJwtPresentationBuilder {
            mut sd_jwt,
            digests_to_be_disclosed,
            mut nondisclosed,
            ..
        } = self;
        sd_jwt.disclosures = digests_to_be_disclosed
            .into_iter()
            .fold(HashMap::new(), |mut disclosures, digest| {
                let disclosure = nondisclosed.remove(&digest).expect("disclosure should be present");
                disclosures.insert(digest, disclosure);
                disclosures
            });

        UnsignedSdJwtPresentation(sd_jwt)
    }
}

impl UnsignedSdJwtPresentation {
    /// Signs the underlying [`SdJwt`] and returns an SD-JWT presentation containing the issuer signed SD-JWT and
    /// KB-JWT.
    ///
    /// ## Errors
    /// - [`Error::InvalidHasher`] is returned if the provided `hasher`'s algorithm doesn't match the algorithm
    ///   specified by SD-JWT's `_sd_alg` claim. "sha-256" is used if the claim is missing.
    pub async fn sign(
        self,
        key_binding_jwt_builder: KeyBindingJwtBuilder,
        hasher: &impl Hasher,
        signing_key: &impl EcdsaKeySend,
    ) -> Result<SdJwtPresentation> {
        let sd_jwt = self.0;

        let required_hasher = sd_jwt.claims()._sd_alg.as_deref().unwrap_or(SHA_ALG_NAME);
        if required_hasher != hasher.alg_name() {
            return Err(Error::InvalidHasher(format!(
                "hasher \"{}\" was provided, but \"{required_hasher} is required\"",
                hasher.alg_name()
            )));
        }

        let kb_jwt = key_binding_jwt_builder.finish(&sd_jwt, hasher, signing_key).await?;

        let sd_jwt_presentation = SdJwtPresentation {
            sd_jwt,
            key_binding_jwt: kb_jwt.into(),
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
    path: &mut Peekable<I>,
    disclosures: &'a HashMap<String, Disclosure>,
    traversing_array: bool,
) -> Result<Vec<&'a str>>
where
    I: ExactSizeIterator<Item = &'a ClaimPath>,
{
    // Holds all digests that should be disclosed based on the `path`
    let mut digests = vec![];

    // If we are traversing an array, peekable shouldn't consume the next value
    let (element_key, has_next) = if traversing_array {
        (*path.peek().ok_or(Error::EmptyPath)?, path.len() > 1)
    } else {
        (path.next().ok_or(Error::EmptyPath)?, path.peek().is_some())
    };

    match (object, element_key) {
        // We are just traversing to a deeper part of the object.
        (serde_json::Value::Object(object), ClaimPath::SelectByKey(key)) if has_next => {
            // Either the element is non-selectively disclosable and present in the object, or it is selectively
            // disclosable and its digest has to be found.
            let next_object = object
                .get(key)
                .or_else(|| {
                    find_disclosure_digest_in_object(object, key, disclosures)
                        .and_then(|digest| {
                            // We're disclosing something within the current object, which is selectively disclosable.
                            // For the verifier to be able to verify that, we'll also have to disclose the current
                            // object.
                            digests.push(digest);
                            disclosures.get(digest)
                        })
                        .map(|disclosure| disclosure.claim_value())
                })
                .ok_or_else(|| Error::IntermediateElementNotFound { path: key.clone() })?;

            digests.append(&mut digests_to_disclose(next_object, path, disclosures, false)?);
            Ok(digests)
        }
        // We reached the the value we want to disclose, so add it to the list of digests
        (serde_json::Value::Object(object), ClaimPath::SelectByKey(key)) => {
            let digest = find_disclosure_digest_in_object(object, key, disclosures)
                .ok_or_else(|| Error::ElementNotFound { path: key.clone() })?;

            digests.push(digest);
            Ok(digests)
        }
        // Traversing an array
        (serde_json::Value::Array(arr), ClaimPath::SelectByIndex(index)) if has_next => {
            let next_object = arr
                .get(*index)
                .and_then(|entry| process_array_entry(entry, disclosures, &mut digests))
                .ok_or_else(|| Error::ElementNotFoundInArray {
                    path: element_key.to_string(),
                })?;

            digests.append(&mut digests_to_disclose(next_object, path, disclosures, false)?);
            Ok(digests)
        }
        // Disclosing an array's entry.
        (serde_json::Value::Array(arr), ClaimPath::SelectByIndex(index)) => {
            let digest = arr
                .get(*index)
                .and_then(|entry| entry.as_object())
                .and_then(|object| find_disclosure_digest_in_array(object))
                .ok_or_else(|| Error::ElementNotFoundInArray {
                    path: element_key.to_string(),
                })?;

            digests.push(digest);
            Ok(digests)
        }
        // Disclosing all array entries
        (serde_json::Value::Array(arr), ClaimPath::SelectAll) => {
            for entry in arr {
                let next_object = process_array_entry(entry, disclosures, &mut digests).ok_or_else(|| {
                    Error::ElementNotFoundInArray {
                        path: element_key.to_string(),
                    }
                })?;

                if has_next {
                    digests.append(&mut digests_to_disclose(next_object, path, disclosures, true)?);
                }
            }

            Ok(digests)
        }
        (element, _) => Err(Error::UnexpectedElement(element.clone(), path.cloned().collect_vec())),
    }
}

fn process_array_entry<'a>(
    entry: &'a serde_json::Value,
    disclosures: &'a HashMap<String, Disclosure>,
    digests: &mut Vec<&'a str>,
) -> Option<&'a serde_json::Value> {
    entry
        .as_object()
        .and_then(|object| find_disclosure_digest_in_array(object))
        .and_then(|digest| {
            // We're disclosing something within a selectively disclosable array entry.
            // For the verifier to be able to verify that, we'll also have to disclose that entry.
            digests.push(digest);
            disclosures.get(digest)
        })
        .map(move |disclosure| disclosure.claim_value())
}

/// Find the digest of the given `key` in the `object` and `disclosures`.
fn find_disclosure_digest_in_object<'o>(
    object: &'o serde_json::Map<String, serde_json::Value>,
    key: &str,
    disclosures: &HashMap<String, Disclosure>,
) -> Option<&'o str> {
    // Try to find the digest for disclosable property `key` in
    // the `_sd` field of `object`.
    object
        .get(DIGESTS_KEY)
        .map(|value| value.as_array().expect("`_sd` must be an array"))
        .iter()
        .flat_map(|values| values.iter())
        .map(|value| value.as_str().expect("digest values should be strings"))
        .find(|digest| {
            disclosures
                .get(*digest)
                .and_then(|disclosure| match &disclosure.content {
                    DisclosureContent::ObjectProperty(_, name, _) => Some(name),
                    _ => None,
                })
                .is_some_and(|name| name == key)
        })
}

/// Find the digest of the given `key` in the `object` and `disclosures`.
fn find_disclosure_digest_in_array(object: &serde_json::Map<String, serde_json::Value>) -> Option<&str> {
    // Try checking `object` as a disclosable array entry.
    object
        .get(ARRAY_DIGEST_KEY)
        .map(|value| value.as_str().expect("digest values should be strings"))
        .filter(|_| object.len() == 1)
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
    use crate::key_binding_jwt_claims::KeyBindingJwtBuilder;
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

        let (expected_jwt, expected_disclosures) =
            SdJwt::parse_sd_jwt_unverified(SIMPLE_STRUCTURED_SD_JWT, &Sha256Hasher).unwrap();

        assert_eq!(sd_jwt.disclosures(), &expected_disclosures);
        assert_eq!(
            sd_jwt.issuer_signed_jwt.payload(),
            &expected_jwt.dangerous_parse_unverified().unwrap().1
        );
    }

    fn create_presentation(
        object: serde_json::Value,
        conceal_paths: &[Vec<&str>],
        disclose_paths: &[Vec<&str>],
    ) -> SdJwtPresentation {
        let issuer_privkey = SigningKey::random(&mut OsRng);
        let holder_privkey = SigningKey::random(&mut OsRng);

        let sd_jwt = conceal_paths
            .iter()
            .fold(SdJwtBuilder::new(object).unwrap(), |builder, path| {
                builder
                    .make_concealable(
                        path.iter()
                            .map(|p| p.parse().unwrap())
                            .collect_vec()
                            .try_into()
                            .unwrap(),
                    )
                    .unwrap()
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
            .fold(sd_jwt.into_presentation_builder(), |builder, path| {
                builder
                    .disclose(
                        &path
                            .iter()
                            .map(|key| key.parse().unwrap())
                            .collect_vec()
                            .try_into()
                            .unwrap(),
                    )
                    .unwrap()
            })
            .finish()
            .sign(
                KeyBindingJwtBuilder::new(Utc::now(), "aud".to_string(), "nonce".to_string(), Algorithm::ES256),
                &Sha256Hasher,
                &holder_privkey,
            )
            .now_or_never()
            .unwrap()
            .unwrap()
    }

    #[rstest]
    #[case::default_nothing_disclosed(
        json!({"given_name": "John", "family_name": "Doe"}),
        &[vec!["given_name"], vec!["family_name"]],
        &[],
        &[],
        &[],
    )]
    #[case::flat_sd_all_disclose_single(
        json!({"given_name": "John", "family_name": "Doe"}),
        &[vec!["given_name"], vec!["family_name"]],
        &[vec!["given_name"]],
        &["given_name"],
        &[],
    )]
    #[case::flat_sd_all_disclose_all(
        json!({"given_name": "John", "family_name": "Doe"}),
        &[vec!["given_name"], vec!["family_name"]],
        &[vec!["given_name"], vec!["family_name"]],
        &["given_name", "family_name"],
        &[],
    )]
    #[case::flat_single_sd(
        json!({"given_name": "John", "family_name": "Doe"}),
        &[vec!["given_name"]],
        &[vec!["given_name"]],
        &["given_name"],
        &["/family_name"],
    )]
    #[case::flat_no_sd_no_disclose(
        json!({"given_name": "John", "family_name": "Doe"}),
        &[],
        &[],
        &[],
        &["/family_name", "/given_name"],
    )]
    #[case::structured_single_sd_and_disclose(
        json!({"address": {"street": "Main st.", "house_number": 4 }}),
        &[vec!["address", "street"]],
        &[vec!["address", "street"]],
        &["street"],
        &["/address", "/address/house_number"],
    )]
    #[case::structured_recursive_path_sd_and_single_disclose(
        json!({"address": {"street": "Main st.", "house_number": 4 }}),
        &[vec!["address", "street"], vec!["address"]],
        &[vec!["address", "street"]],
        &["address", "street"],
        &[],
    )]
    #[case::structured_all_sd_and_all_disclose(
        json!({"address": {"street": "Main st.", "house_number": 4 }}),
        &[vec!["address", "street"], vec!["address", "house_number"], vec!["address"]],
        &[vec!["address", "street"], vec!["address", "house_number"]],
        &["street", "house_number", "address"],
        &[],
    )]
    #[case::structured_all_sd_and_single_disclose(
        json!({"address": {"street": "Main st.", "house_number": 4 }}),
        &[vec!["address", "street"], vec!["address", "house_number"], vec!["address"]],
        &[vec!["address", "street"]],
        &["address", "street"],
        &[],
    )]
    #[case::structured_root_sd_and_root_disclose(
        json!({"address": {"street": "Main st.", "house_number": 4 }}),
        &[vec!["address"]],
        &[vec!["address"]],
        &["address"],
        &[],
    )]
    #[case::array(
        json!({"nationalities": ["NL", "DE"]}),
        &[vec!["nationalities"]],
        &[vec!["nationalities"]],
        &["nationalities"],
        &[],
    )]
    fn test_object_selectively_disclosable_attributes_in_presentation(
        #[case] object: serde_json::Value,
        #[case] conceal_paths: &[Vec<&str>],
        #[case] disclose_paths: &[Vec<&str>],
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
            HashSet::from_iter(expected_disclosed_paths.iter().map(|path| String::from(*path))),
            presentation
                .sd_jwt
                .disclosures
                .into_iter()
                .filter_map(|(_, disclosure)| match disclosure.content {
                    DisclosureContent::ObjectProperty(_, name, _) => Some(name),
                    _ => None,
                })
                .collect::<HashSet<_>>(),
        );

        assert_eq!(
            expected_not_selectively_disclosable_paths
                .iter()
                .map(|path| String::from(*path))
                .collect::<HashSet<_>>(),
            not_selectively_disclosable_paths
        );
    }

    #[rstest]
    #[case::array(
        json!({"nationalities": ["NL", "DE"]}),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"]],
        &[vec!["nationalities", "null"]],
        &["NL", "DE"],
        &["/nationalities"],
    )]
    #[case::array(
        json!({"nationalities": ["NL", "DE"]}),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"], vec!["nationalities"]],
        &[vec!["nationalities", "null"]],
        &["nationalities", "NL", "DE"],
        &[],
    )]
    #[case::array(
        json!({"nationalities": ["NL", "DE"]}),
        &[vec!["nationalities", "0"]],
        &[vec!["nationalities", "0"]],
        &["NL"],
        &["/nationalities/DE", "/nationalities"],
    )]
    #[case::array(
        json!({"nationalities": [{"country": "NL"}, {"country": "DE"}]}),
        &[
            vec!["nationalities", "0", "country"],
            vec!["nationalities", "1", "country"],
            vec!["nationalities", "0"],
            vec!["nationalities", "1"],
            vec!["nationalities"]
        ],
        &[vec!["nationalities", "null", "country"]],
        &["nationalities", "country"],
        &[],
    )]
    #[case::array(
        json!({"nationalities": [{"country": "NL"}, {"country": "DE"}]}),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"], vec!["nationalities"]],
        &[vec!["nationalities", "null"]],
        &["nationalities", "country"],
        &[],
    )]
    #[case::array(
        json!({"nationalities": ["NL", "DE"]}),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"]],
        &[vec!["nationalities", "1"]],
        &["DE"],
        &["/nationalities"],
    )]
    #[case::array(
        json!({"nationalities": ["NL", "DE"]}),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"], vec!["nationalities"]],
        &[vec!["nationalities", "1"]],
        &["nationalities", "DE"],
        &[],
    )]
    #[case::array(
        json!({"nationalities": [{"country": "NL"}, {"country": "DE"}]}),
        &[
            vec!["nationalities", "0", "country"],
            vec!["nationalities", "1", "country"],
            vec!["nationalities", "0"],
            vec!["nationalities", "1"],
            vec!["nationalities"]
        ],
        &[vec!["nationalities", "1", "country"]],
        &["nationalities", "country"],
        &[],
    )]
    #[case::array(
        json!({"nationalities": [{"country": "NL"}, {"country": "DE"}]}),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"], vec!["nationalities"]],
        &[vec!["nationalities", "1"]],
        &["nationalities", "country"],
        &[],
    )]
    fn test_array_selectively_disclosable_attributes_in_presentation(
        #[case] object: serde_json::Value,
        #[case] conceal_paths: &[Vec<&str>],
        #[case] disclose_paths: &[Vec<&str>],
        #[case] expected_disclosed_paths_or_values: &[&str],
        #[case] expected_not_selectively_disclosable_paths_or_values: &[&str],
    ) {
        let presentation = create_presentation(object, conceal_paths, disclose_paths);

        fn get_paths(object: &serde_json::Map<String, serde_json::Value>) -> HashSet<String> {
            fn traverse(value: &serde_json::Value, current_path: &str, paths: &mut HashSet<String>) {
                match value {
                    serde_json::Value::Object(map) => {
                        for (key, val) in map {
                            let new_path = if current_path.is_empty() {
                                format!("/{key}")
                            } else {
                                format!("{current_path}/{key}")
                            };

                            if key != "_sd" && key != "..." {
                                paths.insert(new_path.clone());
                                match val {
                                    serde_json::Value::Object(_) => traverse(val, &new_path, paths),
                                    serde_json::Value::Array(values) => {
                                        values.iter().for_each(|value| traverse(value, &new_path, paths))
                                    }
                                    serde_json::Value::String(s) => {
                                        paths.insert(s.clone());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    serde_json::Value::String(s) => {
                        let new_path = if current_path.is_empty() {
                            format!("/{s}")
                        } else {
                            format!("{current_path}/{s}")
                        };
                        paths.insert(new_path);
                    }
                    _ => {}
                }
            }

            let mut paths = HashSet::new();
            traverse(&serde_json::Value::Object(object.clone()), "", &mut paths);
            paths
        }

        let claims = presentation.sd_jwt.issuer_signed_jwt.payload();
        let not_selectively_disclosable_paths = get_paths(&claims.properties);

        let mut actual_disclosed_paths_or_values = HashSet::new();

        for (_digest, disclosure) in presentation.sd_jwt.disclosures {
            match disclosure.content {
                DisclosureContent::ObjectProperty(_, name, _) => {
                    actual_disclosed_paths_or_values.insert(name);
                }
                DisclosureContent::ArrayElement(_, value) => match value {
                    serde_json::Value::Object(map) => {
                        for (key, _value) in map {
                            if key != "_sd" {
                                actual_disclosed_paths_or_values.insert(key.clone());
                            }
                        }
                    }
                    serde_json::Value::String(value) => {
                        actual_disclosed_paths_or_values.insert(value);
                    }
                    _ => {}
                },
            }
        }

        assert_eq!(
            HashSet::from_iter(
                expected_disclosed_paths_or_values
                    .iter()
                    .map(|path| String::from(*path))
            ),
            actual_disclosed_paths_or_values
        );

        assert_eq!(
            expected_not_selectively_disclosable_paths_or_values
                .iter()
                .map(|path| String::from(*path))
                .collect::<HashSet<_>>(),
            not_selectively_disclosable_paths
        );
    }
}
