// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::LazyLock;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use derive_more::AsRef;
use derive_more::Display;
use itertools::Itertools;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Validation;
use jwt::JwtTyp;
use nutype::nutype;
use p256::ecdsa::VerifyingKey;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Number;
use serde_with::DeserializeFromStr;
use serde_with::FromInto;
use serde_with::SerializeDisplay;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use ssri::Integrity;

use attestation_types::claim_path::ClaimPath;
use crypto::CredentialEcdsaKey;
use crypto::EcdsaKey;
use crypto::wscd::DisclosureWscd;
use crypto::wscd::WscdPoa;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateUsage;
use http_utils::urls::HttpsUri;
use jwt::EcdsaDecodingKey;
use jwt::Header;
use jwt::UnverifiedJwt;
use jwt::VerifiedJwt;
use jwt::headers::HeaderWithX5c;
use utils::date_time_seconds::DateTimeSeconds;
use utils::generator::Generator;
use utils::spec::SpecOptional;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_at_least::VecNonEmptyUnique;

use crate::decoder::SdObjectDecoder;
use crate::disclosure::Disclosure;
use crate::disclosure::DisclosureContent;
use crate::encoder::ARRAY_DIGEST_KEY;
use crate::encoder::DIGESTS_KEY;
use crate::error::Error;
use crate::error::Result;
use crate::hasher::Hasher;
use crate::hasher::Sha256Hasher;
use crate::key_binding_jwt_claims::KeyBindingJwt;
use crate::key_binding_jwt_claims::KeyBindingJwtBuilder;
use crate::key_binding_jwt_claims::RequiredKeyBinding;
use crate::sd_alg::SdAlg;

/// An SD-JWT that has been split into parts but not verified yet. There's no need to keep the SD JWT as serialized form
/// as there is no KB-JWT
#[derive(Debug, Clone, SerializeDisplay, DeserializeFromStr)]
pub struct UnverifiedSdJwt<C = SdJwtClaims, H = HeaderWithX5c> {
    issuer_signed: UnverifiedJwt<C, H>,
    disclosures: Vec<String>,
}

impl<C, H> Display for UnverifiedSdJwt<C, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}~", self.issuer_signed)?;
        for d in &self.disclosures {
            write!(f, "{d}~")?;
        }

        Ok(())
    }
}

impl<C, H> FromStr for UnverifiedSdJwt<C, H> {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.strip_suffix("~").ok_or(Error::Deserialization(
            "SD-JWT format is invalid, input doesn't end with '~'".to_string(),
        ))?;

        let mut segments = s.split('~');
        let issuer_signed = segments
            .next()
            .ok_or(Error::Deserialization(
                "SD-JWT format is invalid, input doesn't contain an issuer signed JWT".to_string(),
            ))?
            .parse()?;
        let disclosures = segments.map(ToString::to_string).collect_vec();

        Ok(UnverifiedSdJwt {
            issuer_signed,
            disclosures,
        })
    }
}

impl UnverifiedSdJwt {
    pub fn into_verified_against_trust_anchors(
        self,
        trust_anchors: &[TrustAnchor],
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<VerifiedSdJwt> {
        let UnverifiedSdJwt {
            issuer_signed,
            disclosures,
        } = self;

        let issuer_signed_jwt = issuer_signed.into_verified_against_trust_anchors(
            &SD_JWT_VALIDATIONS,
            time,
            CertificateUsage::Mdl,
            trust_anchors,
        )?;

        let disclosures = Self::parse_and_verify_disclosures(&disclosures, issuer_signed_jwt.payload())?;

        Ok(VerifiedSdJwt(SdJwt {
            issuer_signed_jwt,
            disclosures,
        }))
    }
}

impl UnverifiedSdJwt {
    /// Parses and verifies disclosures according to <https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-22.html#section-7.1>
    fn parse_and_verify_disclosures(
        disclosures: &[String],
        sd_jwt_claims: &SdJwtClaims,
    ) -> Result<HashMap<String, Disclosure>> {
        let hasher = sd_jwt_claims._sd_alg.unwrap_or_default().hasher()?;

        let mut placeholder_digests: HashMap<String, HashType> = sd_jwt_claims.claims.digests().into_iter().collect();

        let disclosures: HashMap<String, Disclosure> = disclosures
            .iter()
            .map(|disclosure| {
                let hash = hasher.encoded_digest(disclosure);
                let disclosure = disclosure.parse::<Disclosure>()?;

                for (digest, hash_type) in disclosure.content.parsed_claim_value()?.digests() {
                    // 7.1.4. If any digest value is encountered more than once in the Issuer-signed JWT payload
                    // (directly or recursively via other Disclosures), the SD-JWT MUST be rejected.
                    if placeholder_digests.insert(digest.clone(), hash_type).is_some() {
                        return Err(Error::DuplicateHash(digest));
                    }
                }

                Result::Ok((hash, disclosure))
            })
            .try_collect()?;

        // 7.1.5. If any Disclosure was not referenced by digest value in the Issuer-signed JWT (directly or recursively
        // via other Disclosures), the SD-JWT MUST be rejected.
        disclosures
            .iter()
            .try_for_each(|(digest, disclosure)| match placeholder_digests.get(digest) {
                // For any disclosure that is referenced, verify that the hash type matches the digest hash type.
                Some(digest_hash_type) if *digest_hash_type != disclosure.hash_type() => {
                    Err(Error::DataTypeMismatch(format!(
                        "Expected an {:?} element, but got an {digest_hash_type:?} element for digest `{digest}`",
                        disclosure.hash_type()
                    )))
                }
                Some(_) => Ok(()),
                None => Err(Error::UnreferencedDisclosure(digest.clone())),
            })?;

        Ok(disclosures)
    }
}

impl From<UnsignedSdJwtPresentation> for UnverifiedSdJwt {
    fn from(presentation: UnsignedSdJwtPresentation) -> Self {
        // TODO we could define `into_disclosures` on `SdJwt` and use that here.
        let UnsignedSdJwtPresentation(sd_jwt) = presentation;

        let issuer_signed = sd_jwt.issuer_signed_jwt.into();
        let disclosures = sd_jwt
            .disclosures
            .into_values()
            .map(|disclosure| disclosure.encoded)
            .collect();

        Self {
            issuer_signed,
            disclosures,
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SdJwtClaims {
    pub _sd_alg: Option<SdAlg>,

    // TODO this should be mandatory
    pub cnf: Option<RequiredKeyBinding>,

    // TODO this should be mandatory
    #[serde(rename = "vct#integrity")]
    pub vct_integrity: Option<Integrity>,

    // TODO this should be mandatory
    pub vct: Option<String>,

    pub iss: HttpsUri,

    pub iat: DateTimeSeconds,

    pub exp: Option<DateTimeSeconds>,

    pub nbf: Option<DateTimeSeconds>,

    #[serde(flatten)]
    pub claims: ObjectClaims,
}

#[nutype(validate(predicate = |name| !["...", "_sd"].contains(&name)), derive(Debug, Clone, TryFrom, FromStr, PartialEq, Eq, Hash, Serialize, Deserialize))]
pub struct ClaimName(String);

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct ObjectClaims {
    /// Selectively disclosable claims of the SD-JWT.
    pub _sd: Option<VecNonEmptyUnique<String>>,

    /// Non-selectively disclosable claims of the SD-JWT.
    #[serde(flatten)]
    pub claims: HashMap<ClaimName, ClaimValue>,
}

impl ObjectClaims {
    pub fn digests(&self) -> Vec<(String, HashType)> {
        let object_digests = self
            ._sd
            .iter()
            .flat_map(|digests| digests.iter().map(|digest| (digest.clone(), HashType::Object)));

        self.claims
            .values()
            .flat_map(ClaimValue::digests)
            .chain(object_digests)
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum ClaimValue {
    Array(Vec<ArrayClaim>),
    Object(ObjectClaims),
    Null,
    Bool(bool),
    Number(Number),
    String(String),
}

impl ClaimValue {
    /// Recursively discover all placeholder digests for arrays and objects.
    pub fn digests(&self) -> Vec<(String, HashType)> {
        match self {
            ClaimValue::Array(claims) => claims.iter().flat_map(ArrayClaim::digests).collect(),
            ClaimValue::Object(object) => object.digests(),
            // There are no digests in any primitive value.
            _ => Default::default(),
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum ArrayClaim {
    Hash(#[serde_as(as = "FromInto<DisclosureHash>")] String),
    Value(ClaimValue),
}

impl ArrayClaim {
    pub fn digests(&self) -> Vec<(String, HashType)> {
        match &self {
            ArrayClaim::Hash(hash) => vec![(hash.clone(), HashType::Array)],
            ArrayClaim::Value(value) => value.digests(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct DisclosureHash {
    #[serde(rename = "...")]
    hash: String,
}

impl From<String> for DisclosureHash {
    fn from(hash: String) -> Self {
        Self { hash }
    }
}

impl From<DisclosureHash> for String {
    fn from(value: DisclosureHash) -> Self {
        value.hash
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashType {
    Array,
    Object,
}

/// Representation of an SD-JWT of the format
/// `<Issuer-signed JWT>~<Disclosure 1>~<Disclosure 2>~...~<Disclosure N>~`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SdJwt<C = SdJwtClaims, H = HeaderWithX5c> {
    issuer_signed_jwt: VerifiedJwt<C, H>,
    disclosures: HashMap<String, Disclosure>,
}

#[derive(Debug, Clone, Eq, PartialEq, AsRef, Display)]
pub struct VerifiedSdJwt<C = SdJwtClaims, H = HeaderWithX5c>(SdJwt<C, H>);

impl VerifiedSdJwt {
    #[cfg(feature = "test")]
    pub fn new_dangerous(sd_jwt: SdJwt) -> Self {
        Self(sd_jwt)
    }

    pub fn into_inner(self) -> SdJwt {
        self.0
    }
}

#[derive(Debug, Clone, Eq, PartialEq, AsRef)]
pub struct UnsignedSdJwtPresentation<C = SdJwtClaims, H = HeaderWithX5c>(SdJwt<C, H>);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SdJwtPresentation<C = SdJwtClaims, H = HeaderWithX5c> {
    sd_jwt: SdJwt<C, H>,
    key_binding_jwt: SpecOptional<KeyBindingJwt>,
}

impl<H, E> SdJwtPresentation<SdJwtClaims, H>
where
    H: TryFrom<Header, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    /// Create multiple `SdJwtPresentation`s by having the WSCD sign multiple `UnsignedSdJwtPresentation`s,
    /// using the contents of a single `KeyBindingJwtBuilder`.
    pub async fn sign_multiple<I, W, K, P>(
        unsigned_presentations_and_keys_ids: VecNonEmpty<(UnsignedSdJwtPresentation<SdJwtClaims, H>, I)>,
        key_binding_jwt_builder: KeyBindingJwtBuilder,
        wscd: &W,
        poa_input: P::Input,
    ) -> Result<(VecNonEmpty<SdJwtPresentation<SdJwtClaims, H>>, Option<P>)>
    where
        I: Into<String>,
        W: DisclosureWscd<Key = K, Poa = P>,
        K: CredentialEcdsaKey,
        P: WscdPoa,
    {
        // Create the WSCD keys from the provided key identifiers and public keys present in the `cnf` claim.
        // Note that the latter is not actually used, as all we do is signing.
        let sd_jwts_and_keys: VecNonEmpty<(SdJwt<SdJwtClaims, H>, _)> = unsigned_presentations_and_keys_ids
            .into_nonempty_iter()
            .map(|(UnsignedSdJwtPresentation(sd_jwt), key_identifier)| {
                let key = wscd.new_key(key_identifier, sd_jwt.verifying_key()?);

                Ok((sd_jwt, key))
            })
            .collect::<Result<_>>()?;

        // Have the WSCD create `KeyBindingJwt`s and the PoA, if required.
        let (key_binding_jwts, poa) = key_binding_jwt_builder
            .finish_multiple(&sd_jwts_and_keys, wscd, poa_input)
            .await?;

        // Combine the `SdJwt`s with the `KeyBindingJwt`s to create `SdJwtPresentation`s.
        let sd_jwt_presentations = sd_jwts_and_keys
            .into_nonempty_iter()
            .zip(key_binding_jwts)
            .map(|((sd_jwt, _), key_binding_jwt)| SdJwtPresentation {
                sd_jwt,
                key_binding_jwt: key_binding_jwt.into(),
            })
            .collect();

        Ok((sd_jwt_presentations, poa))
    }
}

impl SdJwtPresentation {
    /// Parses an SD-JWT into its components as [`SdJwtPresentation`] while verifying against a set of trust anchors.
    ///
    /// ## Error
    /// Returns [`Error::Deserialization`] if parsing fails.
    pub fn parse_and_verify_against_trust_anchors(
        sd_jwt: &str,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
        kb_expected_aud: &str,
        kb_expected_nonce: &str,
        kb_iat_acceptance_window: Duration,
    ) -> Result<SdJwtPresentation> {
        let (rest, kb_segment) = Self::split_sd_jwt_kb(sd_jwt)?;

        let verified_sd_jwt = VerifiedSdJwt::parse_and_verify_against_trust_anchors(rest, time, trust_anchors)?;

        let key_binding_jwt = SdJwtPresentation::parse_and_verify_kb_jwt(
            kb_segment,
            verified_sd_jwt.as_ref(),
            kb_expected_aud,
            kb_expected_nonce,
            kb_iat_acceptance_window,
            time,
        )?;

        Ok(SdJwtPresentation {
            sd_jwt: verified_sd_jwt.into_inner(),
            key_binding_jwt: key_binding_jwt.into(),
        })
    }
}

impl<C, H> SdJwtPresentation<C, H> {
    pub fn presentation(&self) -> String {
        let disclosures = self.sd_jwt.disclosures.values().join("~");
        let key_bindings = self.key_binding_jwt.as_ref().to_string();
        [
            self.sd_jwt.issuer_signed_jwt.jwt().to_string(),
            disclosures,
            key_bindings,
        ]
        .into_iter()
        .filter(|segment| !segment.is_empty())
        .join("~")
    }

    fn split_sd_jwt_kb(sd_jwt: &str) -> Result<(&str, &str)> {
        sd_jwt
            .rsplit_once("~")
            .map(|(head, tail)| {
                let head_with_tilde = &sd_jwt[..head.len() + 1];
                (head_with_tilde, tail)
            })
            .ok_or(Error::Deserialization(
                "SD-JWT format is invalid, no segments found".to_string(),
            ))
    }
}

impl SdJwtPresentation {
    pub fn sd_jwt(&self) -> &SdJwt {
        &self.sd_jwt
    }

    pub fn into_sd_jwt(self) -> SdJwt {
        self.sd_jwt
    }

    pub fn key_binding_jwt(&self) -> &KeyBindingJwt {
        self.key_binding_jwt.as_ref()
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        // TODO (PVW-4817): The presence of the key binding and the guarantee that it can be parsed to a `VerifyingKey`
        //                  is inherent to the type, as this should already have been done by the two ways to construct
        //                  it:
        //
        //                  * When the holder creates this type through `SdJwtPresentationBuilder` by signing a
        //                    `KeyBindingJwt` with its private key.
        //                  * When the verifier parses the type from a SD-JWT presentation string.
        //
        //                  Unfortunately the presence and validity of the public key is currently not checked for the
        //                  first method. This sanity check should be added, so we know the guarantee holds.
        self.sd_jwt.verifying_key().unwrap()
    }
}

impl<H> Display for SdJwtPresentation<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&(self.presentation()))
    }
}

impl<C, H> SdJwt<C, H> {
    /// Creates a new [`SdJwt`] from its components.
    pub(crate) fn new(issuer_signed_jwt: VerifiedJwt<C, H>, disclosures: HashMap<String, Disclosure>) -> Self {
        Self {
            issuer_signed_jwt,
            disclosures,
        }
    }

    pub fn claims(&self) -> &C {
        self.issuer_signed_jwt.payload()
    }

    pub fn disclosures(&self) -> &HashMap<String, Disclosure> {
        &self.disclosures
    }

    /// Serializes the components into the final SD-JWT.
    pub fn presentation(&self) -> String {
        let disclosures = self.disclosures.values().join("~");
        if disclosures.is_empty() {
            format!("{}~", self.issuer_signed_jwt.jwt())
        } else {
            format!("{}~{}~", self.issuer_signed_jwt.jwt(), disclosures)
        }
    }

    /// Returns the JSON object obtained by replacing all disclosures into their
    /// corresponding JWT concealable claims.
    pub fn to_disclosed_object(&self) -> Result<serde_json::Map<String, serde_json::Value>>
    where
        C: Serialize,
    {
        let decoder = SdObjectDecoder;
        let object = serde_json::to_value(self.claims())?;

        decoder.decode(object.as_object().unwrap(), &self.disclosures)
    }
}

impl SdJwt {
    /// Prepares this [`SdJwt`] for a presentation, returning an [`SdJwtPresentationBuilder`].
    pub fn into_presentation_builder(self) -> SdJwtPresentationBuilder {
        SdJwtPresentationBuilder::new(self)
    }
}

impl<C> SdJwt<C> {
    pub fn issuer_certificate_chain(&self) -> &VecNonEmpty<BorrowingCertificate> {
        &self.issuer_signed_jwt.header().x5c
    }

    pub fn issuer_certificate(&self) -> &BorrowingCertificate {
        // From https://datatracker.ietf.org/doc/html/rfc7515:
        // The certificate containing the public key corresponding to the key used to digitally sign the
        // JWS MUST be the first certificate.
        self.issuer_signed_jwt.header().x5c.first()
    }
}

impl<H> SdJwt<SdJwtClaims, H> {
    pub fn required_key_bind(&self) -> Option<&RequiredKeyBinding> {
        self.claims().cnf.as_ref()
    }

    pub fn verifying_key(&self) -> Result<VerifyingKey> {
        let verifying_key = self
            .required_key_bind()
            .ok_or(Error::MissingJwkKeybinding)?
            .verifying_key()?;

        Ok(verifying_key)
    }

    pub fn hasher(&self) -> Result<Box<dyn Hasher>> {
        let alg = self.claims()._sd_alg.unwrap_or_default();
        Ok(Box::new(alg.hasher()?))
    }
}

impl<C, H, E> SdJwt<C, H>
where
    C: DeserializeOwned + JwtTyp,
    H: TryFrom<Header, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    /// Parses an SD-JWT into its components as [`SdJwt`].
    ///
    /// ## Error
    /// Returns [`Error::Deserialization`] if parsing fails.
    pub fn parse_and_verify(sd_jwt: &str, pubkey: &EcdsaDecodingKey) -> Result<Self> {
        let (jwt, disclosures) = Self::parse_sd_jwt_unverified(sd_jwt)?;

        let issuer_signed_jwt = jwt.into_verified(pubkey, &SD_JWT_VALIDATIONS)?;

        Ok(Self {
            issuer_signed_jwt,
            disclosures,
        })
    }

    fn parse_sd_jwt_unverified(sd_jwt: &str) -> Result<(UnverifiedJwt<C, H>, HashMap<String, Disclosure>)> {
        if !sd_jwt.ends_with("~") {
            return Err(Error::Deserialization(
                "SD-JWT format is invalid, input doesn't end with '~'".to_string(),
            ));
        }

        let (sd_jwt_segment, disclosure_segments) = sd_jwt.split_once('~').ok_or(Error::Deserialization(
            "SD-JWT format is invalid, input doesn't contain a '~'".to_string(),
        ))?;

        let jwt = sd_jwt_segment.parse()?;

        // TODO first parse the JWT, then get the hasher from the JWT (PVW-4817)
        let hasher = Sha256Hasher;
        let disclosures = disclosure_segments
            .split("~")
            .filter(|segment| !segment.is_empty())
            .try_fold(HashMap::new(), |mut acc, segment| {
                let disclosure: Disclosure = segment.parse()?;

                // Verify disclosure value by parsing it as [ClaimValue].
                // TODO: Use [ClaimValue] internally in [Disclosure] (PVW-4843)
                serde_json::from_value::<ClaimValue>(disclosure.content.claim_value().clone())?;

                acc.insert(hasher.encoded_digest(disclosure.as_str()), disclosure);
                Ok::<_, Error>(acc)
            })?;

        Ok((jwt, disclosures))
    }
}

impl<H, E> SdJwtPresentation<SdJwtClaims, H>
where
    H: TryFrom<Header, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    /// Parses an SD-JWT into its components as [`SdJwtPresentation`].
    ///
    /// ## Error
    /// Returns [`Error::Deserialization`] if parsing fails.
    pub fn parse_and_verify(
        sd_jwt: &str,
        issuer_pubkey: &EcdsaDecodingKey,
        kb_expected_aud: &str,
        kb_expected_nonce: &str,
        kb_iat_acceptance_window: Duration,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<Self> {
        let (rest, kb_segment) = SdJwtPresentation::<SdJwtClaims, H>::split_sd_jwt_kb(sd_jwt)?;

        let sd_jwt = SdJwt::<SdJwtClaims, H>::parse_and_verify(rest, issuer_pubkey)?;

        let key_binding_jwt = SdJwtPresentation::parse_and_verify_kb_jwt(
            kb_segment,
            &sd_jwt,
            kb_expected_aud,
            kb_expected_nonce,
            kb_iat_acceptance_window,
            time,
        )?;

        Ok(Self {
            sd_jwt,
            key_binding_jwt: key_binding_jwt.into(),
        })
    }

    fn parse_and_verify_kb_jwt(
        kb_segment: &str,
        sd_jwt: &SdJwt<SdJwtClaims, H>,
        kb_expected_aud: &str,
        kb_expected_nonce: &str,
        kb_iat_acceptance_window: Duration,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<KeyBindingJwt> {
        KeyBindingJwt::parse_and_verify(
            kb_segment,
            &EcdsaDecodingKey::from(&sd_jwt.verifying_key()?),
            kb_expected_aud,
            kb_expected_nonce,
            kb_iat_acceptance_window,
            time,
        )
    }
}

impl<C, H> Display for SdJwt<C, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&(self.presentation()))
    }
}

impl<C: DeserializeOwned + JwtTyp> VerifiedSdJwt<C> {
    /// Parses an SD-JWT into its components as [`VerifiedSdJwt`] verifying against the provided trust anchors.
    pub fn parse_and_verify_against_trust_anchors(
        sd_jwt: &str,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<Self> {
        let (jwt, disclosures) = SdJwt::<C>::parse_sd_jwt_unverified(sd_jwt)?;

        let issuer_signed_jwt =
            jwt.into_verified_against_trust_anchors(&SD_JWT_VALIDATIONS, time, CertificateUsage::Mdl, trust_anchors)?;

        Ok(Self(SdJwt {
            issuer_signed_jwt,
            disclosures,
        }))
    }
}

impl VerifiedSdJwt {
    /// Parses an SD-JWT into its components as [`VerifiedSdJwt`] without verifying the signature.
    ///
    /// ## Error
    /// Returns [`Error::Deserialization`] if parsing fails.
    pub fn dangerous_parse_unverified(sd_jwt: &str) -> Result<Self> {
        let (jwt, disclosures) = SdJwt::<SdJwtClaims, HeaderWithX5c>::parse_sd_jwt_unverified(sd_jwt)?;

        let issuer_signed_jwt = VerifiedJwt::dangerous_parse_unverified(jwt.serialization())?;

        Ok(Self(SdJwt {
            issuer_signed_jwt,
            disclosures,
        }))
    }
}

impl VerifiedSdJwt {
    pub fn issuer_certificate(&self) -> &BorrowingCertificate {
        self.0.issuer_certificate()
    }

    pub fn into_presentation_builder(self) -> SdJwtPresentationBuilder {
        self.0.into_presentation_builder()
    }
}

#[derive(Clone)]
pub struct SdJwtPresentationBuilder<H = HeaderWithX5c> {
    sd_jwt: SdJwt<SdJwtClaims, H>,

    /// Non-disclosed attributes. All attributes start here. Calling `disclose()` moves an attribute from here
    /// to `disclosed`.
    nondisclosed: HashMap<String, Disclosure>,

    /// Digests to be disclosed.
    digests_to_be_disclosed: HashSet<String>,

    /// A helper object containing both non-selectively disclosable JWT claims and the `_sd` digests,
    /// used by `digests_to_disclose()`.
    full_payload: serde_json::Value,
}

impl<H> SdJwtPresentationBuilder<H> {
    pub(crate) fn new(mut sd_jwt: SdJwt<SdJwtClaims, H>) -> Self {
        let payload = sd_jwt.issuer_signed_jwt.payload();
        let full_payload = serde_json::to_value(&payload.claims)
            .expect("should never fail because Serialize is derived on ObjectClaims");

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
            let mut path_segments = path.as_ref().iter().peekable();
            digests_to_disclose(&self.full_payload, &mut path_segments, &self.nondisclosed, false)?
                .into_iter()
                .map(String::from)
        });

        Ok(self)
    }

    pub fn finish(self) -> UnsignedSdJwtPresentation<SdJwtClaims, H> {
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
        signing_key: &impl EcdsaKey,
    ) -> Result<SdJwtPresentation> {
        let sd_jwt = self.0;

        let kb_jwt = key_binding_jwt_builder.finish(&sd_jwt, signing_key).await?;

        let sd_jwt_presentation = SdJwtPresentation {
            sd_jwt,
            key_binding_jwt: kb_jwt.into(),
        };

        Ok(sd_jwt_presentation)
    }
}

pub static SD_JWT_VALIDATIONS: LazyLock<Validation> = LazyLock::new(|| {
    let mut validation = Validation::new(Algorithm::ES256);

    validation.validate_aud = false;
    validation.validate_nbf = true;
    validation.leeway = 0;
    validation.required_spec_claims.clear(); // remove "exp" from required claims

    validation
});

/// Recursively searches for the specified path in the object and disclosures, returning the digests
/// of objects which are to be disclosed in order to disclose the specified `path.`
///
/// The `object` must be the payload of an SD-JWT, containing an `_sd` array and other claims.
fn digests_to_disclose<'a, I>(
    object: &'a serde_json::Value,
    path: &mut std::iter::Peekable<I>,
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
            // If the value exists within the object, it is not selectively disclosable
            // and we do not have to look for the associated disclosure.
            if object.contains_key(key) {
                return Ok(digests);
            }

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
            let Some(entry) = arr.get(*index) else {
                return Err(Error::ElementNotFoundInArray {
                    path: element_key.to_string(),
                });
            };

            // If the array entry does not look exactly like an array-selective-disclosure object, then this
            // entry is not selectively disclosable and we do not have to look for the associated disclosure.
            let digest = entry
                .as_object()
                .and_then(|object| find_disclosure_digest_in_array(object));
            if let Some(digest) = digest {
                digests.push(digest);
            }

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
    let digest = entry
        .as_object()
        .and_then(|object| find_disclosure_digest_in_array(object));

    if let Some(digest) = digest {
        // We're disclosing something within a selectively disclosable array entry.
        // For the verifier to be able to verify that, we'll also have to disclose that entry.
        digests.push(digest);

        disclosures.get(digest).map(|disclosure| disclosure.claim_value())
    } else {
        // This array entry is not selectively disclosable as it does not look like an
        // array-selective-disclosure object, so we just return it verbatim.
        Some(entry)
    }
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

#[cfg(feature = "mock")]
mod mock {
    use super::SdJwt;
    use super::VerifiedSdJwt;

    impl VerifiedSdJwt {
        pub fn new_mock(sd_jwt: SdJwt) -> Self {
            Self(sd_jwt)
        }
    }
}

#[cfg(feature = "examples")]
mod examples {
    use futures::FutureExt;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use serde_json::json;
    use ssri::Integrity;

    use attestation_types::claim_path::ClaimPath;
    use crypto::server_keys::KeyPair;
    use crypto::utils::random_string;

    use crate::builder::SdJwtBuilder;

    use super::VerifiedSdJwt;

    impl VerifiedSdJwt {
        pub fn pid_example(issuer_keypair: &KeyPair) -> Self {
            let object = json!({
                "vct": "urn:eudi:pid:nl:1",
                "iat": 1683000000,
                "exp": 1883000000,
                "iss": "https://cert.issuer.example.com",
                "attestation_qualification": "QEAA",
                "bsn": "999991772",
                "recovery_code": "cff292503cba8c4fbf2e5820dcdc468ae00f40c87b1af35513375800128fc00d",
                "given_name": "John",
                "family_name": "Doe",
                "birthdate": "1940-01-01"
            });

            let holder_privkey = SigningKey::random(&mut OsRng);

            // issuer signs SD-JWT
            let sd_jwt = SdJwtBuilder::new(object)
                .unwrap()
                .make_concealable(
                    vec![ClaimPath::SelectByKey(String::from("family_name"))]
                        .try_into()
                        .unwrap(),
                )
                .unwrap()
                .make_concealable(vec![ClaimPath::SelectByKey(String::from("bsn"))].try_into().unwrap())
                .unwrap()
                .add_decoys(&[], 2)
                .unwrap()
                .finish(
                    Integrity::from(random_string(32)),
                    issuer_keypair,
                    holder_privkey.verifying_key(),
                )
                .now_or_never()
                .unwrap()
                .unwrap();

            Self(sd_jwt)
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::collections::HashSet;

    use assert_matches::assert_matches;
    use chrono::DateTime;
    use chrono::Duration;
    use chrono::Utc;
    use futures::FutureExt;
    use itertools::Itertools;
    use jsonwebtoken::errors::ErrorKind;
    use jsonwebtoken::jwk::AlgorithmParameters;
    use jsonwebtoken::jwk::EllipticCurve;
    use jsonwebtoken::jwk::EllipticCurveKeyParameters;
    use jsonwebtoken::jwk::EllipticCurveKeyType;
    use jsonwebtoken::jwk::Jwk;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use rstest::rstest;
    use serde_json::Value;
    use serde_json::json;
    use ssri::Integrity;

    use crypto::server_keys::generate::Ca;
    use http_utils::urls::HttpsUri;
    use jwt::Header;
    use jwt::error::JwtError;
    use utils::date_time_seconds::DateTimeSeconds;

    use crate::builder::SdJwtBuilder;
    use crate::disclosure::Disclosure;
    use crate::disclosure::DisclosureContent;
    use crate::encoder::ARRAY_DIGEST_KEY;
    use crate::encoder::DIGESTS_KEY;
    use crate::error::Result;
    use crate::examples::KeyBindingExampleTimeGenerator;
    use crate::examples::*;
    use crate::hasher::Hasher;
    use crate::key_binding_jwt_claims::KeyBindingJwtBuilder;
    use crate::key_binding_jwt_claims::RequiredKeyBinding;
    use crate::sd_alg::SdAlg;
    use crate::sd_jwt::ArrayClaim;
    use crate::sd_jwt::ClaimValue;
    use crate::sd_jwt::Error;
    use crate::sd_jwt::ObjectClaims;
    use crate::sd_jwt::SdJwt;
    use crate::sd_jwt::SdJwtClaims;
    use crate::sd_jwt::SdJwtPresentation;
    use crate::sd_jwt::UnverifiedSdJwt;

    #[rstest]
    #[case(SIMPLE_STRUCTURED_SD_JWT)]
    #[case(COMPLEX_STRUCTURED_SD_JWT)]
    fn parse_sd_jwt_example(#[case] encoded_sd_jwt: &str) {
        SdJwt::<SdJwtExampleClaims, Header>::parse_and_verify(encoded_sd_jwt, &examples_sd_jwt_decoding_key()).unwrap();
    }

    #[test]
    fn parse_sd_jwt_vc_example() {
        SdJwt::<SdJwtClaims, Header>::parse_and_verify(SD_JWT_VC, &examples_sd_jwt_decoding_key()).unwrap();
    }

    #[rstest]
    #[case(SIMPLE_STRUCTURED_SD_JWT)]
    #[case(COMPLEX_STRUCTURED_SD_JWT)]
    fn parse_unverified_sd_jwt(#[case] encoded: &str) {
        encoded.parse::<UnverifiedSdJwt<SdJwtExampleClaims, Header>>().unwrap();
    }

    #[test]
    fn parse_unverified_sd_jwt_vc() {
        SD_JWT_VC.parse::<UnverifiedSdJwt<SdJwtClaims, Header>>().unwrap();
    }

    #[test]
    fn parse_kb() {
        SdJwtPresentation::<SdJwtClaims, Header>::parse_and_verify(
            SD_JWT_VC_WITH_KB,
            &examples_sd_jwt_decoding_key(),
            WITH_KB_SD_JWT_AUD,
            WITH_KB_SD_JWT_NONCE,
            Duration::minutes(10),
            &KeyBindingExampleTimeGenerator,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_parse_should_error_for_expired_jwt() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_keypair = ca.generate_issuer_mock().unwrap();

        let holder_privkey = SigningKey::random(&mut OsRng);

        let sd_jwt = SdJwtBuilder::new(json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "exp": (Utc::now() - Duration::days(1)).timestamp(),
        }))
        .unwrap()
        .finish(Integrity::from(""), &issuer_keypair, holder_privkey.verifying_key())
        .await
        .unwrap()
        .to_string();

        let err = SdJwt::<SdJwtExampleClaims, Header>::parse_and_verify(
            &sd_jwt,
            &issuer_keypair.certificate().public_key().into(),
        )
        .expect_err("should fail");

        assert_matches!(err, Error::JwtParsing(JwtError::Validation(err)) if err.kind() == &ErrorKind::ExpiredSignature);
    }

    #[test]
    fn parse() {
        let sd_jwt = SdJwt::<SdJwtExampleClaims, Header>::parse_and_verify(
            SIMPLE_STRUCTURED_SD_JWT,
            &examples_sd_jwt_decoding_key(),
        )
        .unwrap();
        assert_eq!(sd_jwt.disclosures.len(), 2);
    }

    #[test]
    fn parse_vc() {
        let sd_jwt =
            SdJwt::<SdJwtClaims, Header>::parse_and_verify(SD_JWT_VC, &examples_sd_jwt_decoding_key()).unwrap();
        assert_eq!(sd_jwt.disclosures.len(), 21);
    }

    #[test]
    fn round_trip_ser_des() {
        let sd_jwt = SdJwt::<SdJwtExampleClaims, Header>::parse_and_verify(
            SIMPLE_STRUCTURED_SD_JWT,
            &examples_sd_jwt_decoding_key(),
        )
        .unwrap();

        let (expected_jwt, expected_disclosures) =
            SdJwt::<SdJwtExampleClaims, Header>::parse_sd_jwt_unverified(SIMPLE_STRUCTURED_SD_JWT).unwrap();

        assert_eq!(sd_jwt.disclosures(), &expected_disclosures);
        assert_eq!(
            sd_jwt.issuer_signed_jwt.payload(),
            &expected_jwt.dangerous_parse_unverified().unwrap().1
        );
    }

    #[test]
    fn parse_invalid_disclosure() {
        let result = SdJwt::<SdJwtExampleClaims, Header>::parse_and_verify(
            INVALID_DISCLOSURE_SD_JWT.trim(),
            &examples_sd_jwt_decoding_key(),
        );
        assert_matches!(result, Err(crate::error::Error::Serialization(_)));
    }

    fn create_presentation(
        object: serde_json::Value,
        conceal_paths: &[Vec<&str>],
        disclose_paths: &[Vec<&str>],
    ) -> SdJwtPresentation {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_keypair = ca.generate_issuer_mock().unwrap();
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
            .finish(Integrity::from(""), &issuer_keypair, holder_privkey.verifying_key())
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
                KeyBindingJwtBuilder::new(Utc::now(), "aud".to_string(), "nonce".to_string()),
                &holder_privkey,
            )
            .now_or_never()
            .unwrap()
            .unwrap()
    }

    #[rstest]
    #[case::default_nothing_disclosed(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "given_name": "John",
            "family_name": "Doe"
        }),
        &[vec!["given_name"], vec!["family_name"]],
        &[],
        &[],
        &[],
    )]
    #[case::flat_sd_all_disclose_single(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "given_name": "John",
            "family_name": "Doe"
        }),
        &[vec!["given_name"], vec!["family_name"]],
        &[vec!["given_name"]],
        &["given_name"],
        &[],
    )]
    #[case::flat_sd_all_disclose_all(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "given_name": "John",
            "family_name": "Doe"
        }),
        &[vec!["given_name"], vec!["family_name"]],
        &[vec!["given_name"], vec!["family_name"]],
        &["given_name", "family_name"],
        &[],
    )]
    #[case::flat_single_sd(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "given_name": "John",
            "family_name": "Doe"
        }),
        &[vec!["given_name"]],
        &[vec!["given_name"]],
        &["given_name"],
        &["/family_name"],
    )]
    #[case::flat_single_sd_disclose_all(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "given_name": "John",
            "family_name": "Doe"
        }),
        &[vec!["given_name"]],
        &[vec!["given_name"], vec!["family_name"]],
        &["given_name"],
        &["/family_name"],
    )]
    #[case::flat_no_sd_no_disclose(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "given_name": "John",
            "family_name": "Doe"
        }),
        &[],
        &[],
        &[],
        &["/family_name", "/given_name"],
    )]
    #[case::structured_single_sd_and_disclose(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "address": {
                "street": "Main st.",
                "house_number": 4
            }
        }),
        &[vec!["address", "street"]],
        &[vec!["address", "street"]],
        &["street"],
        &["/address", "/address/house_number"],
    )]
    #[case::structured_recursive_path_sd_and_single_disclose(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "address": {
                "street": "Main st.",
                "house_number": 4
            }
        }),
        &[vec!["address", "street"], vec!["address"]],
        &[vec!["address", "street"]],
        &["address", "street"],
        &[],
    )]
    #[case::structured_all_sd_and_all_disclose(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "address": {
                "street": "Main st.",
                "house_number": 4
            }
        }),
        &[vec!["address", "street"], vec!["address", "house_number"], vec!["address"]],
        &[vec!["address", "street"], vec!["address", "house_number"]],
        &["street", "house_number", "address"],
        &[],
    )]
    #[case::structured_all_sd_and_single_disclose(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "address": {
                "street": "Main st.",
                "house_number": 4
            }
        }),
        &[vec!["address", "street"], vec!["address", "house_number"], vec!["address"]],
        &[vec!["address", "street"]],
        &["address", "street"],
        &[],
    )]
    #[case::structured_root_sd_and_root_disclose(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "address": {"street": "Main st.", "house_number": 4 }
        }),
        &[vec!["address"]],
        &[vec!["address"]],
        &["address"],
        &[],
    )]
    #[case::array(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "nationalities": ["NL", "DE"]
        }),
        &[vec!["nationalities"]],
        &[vec!["nationalities"]],
        &["nationalities"],
        &[],
    )]
    fn test_object_selectively_disclosable_attributes_in_presentation(
        #[case] object: serde_json::Value,
        #[case] conceal_paths: &[Vec<&str>],
        #[case] disclose_paths: &[Vec<&str>],
        #[case] expected_disclosure_paths: &[&str],
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
        let serde_json::Value::Object(properties) = serde_json::to_value(&claims.claims).unwrap() else {
            panic!("unexpected")
        };
        let not_selectively_disclosable_paths = get_paths(&properties);

        assert_eq!(
            HashSet::from_iter(expected_disclosure_paths.iter().map(|path| String::from(*path))),
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
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "nationalities": ["NL", "DE"]
        }),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"]],
        &[vec!["nationalities", "null"]],
        &["NL", "DE"],
        &["/nationalities"],
    )]
    #[case::array_all_non_sd(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "nationalities": ["NL", "DE"]
        }),
        &[vec!["nationalities", "1"]],
        &[vec!["nationalities", "null"]],
        &["DE"],
        &["/nationalities/NL", "/nationalities"],
    )]
    #[case::array(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "nationalities": ["NL", "DE"]
        }),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"], vec!["nationalities"]],
        &[vec!["nationalities", "null"]],
        &["nationalities", "NL", "DE"],
        &[],
    )]
    #[case::array(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "nationalities": ["NL", "DE"]
        }),
        &[vec!["nationalities", "0"]],
        &[vec!["nationalities", "0"]],
        &["NL"],
        &["/nationalities/DE", "/nationalities"],
    )]
    #[case::array_index_non_sd(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "nationalities": ["NL", "DE"]
        }),
        &[vec!["nationalities", "0"]],
        &[vec!["nationalities", "0"], vec!["nationalities", "1"]],
        &["NL"],
        &["/nationalities/DE", "/nationalities"],
    )]
    #[case::array(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "nationalities": [{"country": "NL"}, {"country": "DE"}]
        }),
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
    #[case::array_all_non_sd_nested(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "nationalities": [{"country": "NL"}, {"country": "DE"}]
        }),
        &[
            vec!["nationalities", "1", "country"],
            vec!["nationalities", "1"],
            vec!["nationalities"]
        ],
        &[vec!["nationalities", "null", "country"]],
        &["nationalities", "country"],
        &[],
    )]
    #[case::array(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "nationalities": [{"country": "NL"}, {"country": "DE"}]
        }),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"], vec!["nationalities"]],
        &[vec!["nationalities", "null"]],
        &["nationalities", "country"],
        &[],
    )]
    #[case::array(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "nationalities": ["NL", "DE"]
        }),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"]],
        &[vec!["nationalities", "1"]],
        &["DE"],
        &["/nationalities"],
    )]
    #[case::array(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "nationalities": ["NL", "DE"]
        }),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"], vec!["nationalities"]],
        &[vec!["nationalities", "1"]],
        &["nationalities", "DE"],
        &[],
    )]
    #[case::array(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "nationalities": [{"country": "NL"}, {"country": "DE"}]
        }),
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
    #[case::array_index_non_sd_nested(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "nationalities": [{"country": "NL"}, {"country": "DE"}]
        }),
        &[
            vec!["nationalities", "0", "country"],
            vec!["nationalities", "0"],
            vec!["nationalities"]
        ],
        &[vec!["nationalities", "0", "country"], vec!["nationalities", "1", "country"]],
        &["nationalities", "country"],
        &[],
    )]
    #[case::array(
        json!({
            "iss": "https://iss.example.com",
            "iat": Utc::now().timestamp(),
            "nationalities": [{"country": "NL"}, {"country": "DE"}]
        }),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"], vec!["nationalities"]],
        &[vec!["nationalities", "1"]],
        &["nationalities", "country"],
        &[],
    )]
    fn test_array_selectively_disclosable_attributes_in_presentation(
        #[case] object: serde_json::Value,
        #[case] conceal_paths: &[Vec<&str>],
        #[case] disclose_paths: &[Vec<&str>],
        #[case] expected_disclosure_paths_or_values: &[&str],
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

                            if key != DIGESTS_KEY && key != ARRAY_DIGEST_KEY {
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

        let payload = presentation.sd_jwt.issuer_signed_jwt.payload();
        let serde_json::Value::Object(properties) = serde_json::to_value(&payload.claims).unwrap() else {
            panic!("unexpected")
        };
        let not_selectively_disclosable_paths = get_paths(&properties);

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
                expected_disclosure_paths_or_values
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

    #[rstest]
    #[case(json!({
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "given_name": "Alice",
        "_sd": ["X9yH0Ajrdm1Oij4tWso9UzzKJvPoDxwmuEcO3XAdRC0"]
    }), true)]
    #[case(json!({
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "_sd": [0]
    }), false)]
    #[case(json!({
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "nested": {
            "_sd": [0]
        }
    }), false)]
    #[case(json!({
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "array": [{
            "_sd": [0]
        }]
    }), false)]
    #[case(json!({
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "array": [{ "...": 0 }]
    }), false)]
    #[case(json!({
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "...": "not_allowed"
    }), false)]
    #[case(json!({
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "nationalities":
        ["DE", {"...":"w0I8EKcdCtUPkGCNUrfwVp2xEgNjtoIDlOxc9-PlOhs"}, "US"]
    }), true)]
    #[case(json!({
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "family_name": "Möbius",
        "nationalities": [
            { "...": "PmnlrRjhLcwf8zTDdK15HVGwHtPYjddvD362WjBLwro" },
            { "...": "r823HFN6Ba_lpSANYtXqqCBAH-TsQlIzfOK0lRAFLCM" },
            { "...": "nP5GYjwhFm6ESlAeC4NCaIliW4tz0hTrUeoJB3lb5TA" }
        ]
    }), true)]
    #[case(json!({
        "_sd": [
            "CrQe7S5kqBAHt-nMYXgc6bdt2SH5aTY1sU_M-PgkjPI",
            "JzYjH4svliH0R3PyEMfeZu6Jt69u5qehZo7F7EPYlSE",
            "PorFbpKuVu6xymJagvkFsFXAbRoc2JGlAUA2BA4o7cI",
            "TGf4oLbgwd5JQaHyKVQZU9UdGE0w5rtDsrZzfUaomLo",
            "XQ_3kPKt1XyX7KANkqVR6yZ2Va5NrPIvPYbyMvRKBMM",
            "XzFrzwscM6Gn6CJDc6vVK8BkMnfG8vOSKfpPIZdAfdE",
            "gbOsI4Edq2x2Kw-w5wPEzakob9hV1cRD0ATN3oQL9JM",
            "jsu9yVulwQQlhFlM_3JlzMaSFzglhQG0DpfayQwLUK4"
        ],
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "exp": 1883000000,
        "sub": "user_42",
        "nationalities": [
            {
                "...": "pFndjkZ_VCzmyTa6UjlZo3dh-ko8aIKQc9DlGzhaVYo"
            },
            {
                "...": "7Cf6JkPudry3lcbwHgeZ8khAv1U1OSlerP0VkBJrWZ0"
            }
        ],
        "_sd_alg": "sha-256",
        "cnf": {
            "jwk": {
                "kty": "EC",
                "crv": "P-256",
                "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
            }
        }
    }), true)]
    #[case(json!({
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "exp": 1883000000,
        "sub": "6c5c0a49-b589-431d-bae7-219122a9ec2c",
        "address": {
            "_sd": [
                "6vh9bq-zS4GKM_7GpggVbYzzu6oOGXrmNVGPHP75Ud0",
                "9gjVuXtdFROCgRrtNcGUXmF65rdezi_6Er_j76kmYyM",
                "KURDPh4ZC19-3tiz-Df39V8eidy1oV3a3H1Da2N0g88",
                "WN9r9dCBJ8HTCsS2jKASxTjEyW5m5x65_Z_2ro2jfXM"
            ]
        },
        "_sd_alg": "sha-256"
    }), true)]
    #[case(json!({
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "exp": 1883000000,
        "sub": "6c5c0a49-b589-431d-bae7-219122a9ec2c",
        "address": {
            "_sd": [
                "6vh9bq-zS4GKM_7GpggVbYzzu6oOGXrmNVGPHP75Ud0",
                "9gjVuXtdFROCgRrtNcGUXmF65rdezi_6Er_j76kmYyM",
                "KURDPh4ZC19-3tiz-Df39V8eidy1oV3a3H1Da2N0g88"
            ],
            "country": "DE"
        },
        "_sd_alg": "sha-256"
    }
    ), true)]
    #[case(json!({
        "_sd": [
            "-aSznId9mWM8ocuQolCllsxVggq1-vHW4OtnhUtVmWw",
            "IKbrYNn3vA7WEFrysvbdBJjDDU_EvQIr0W18vTRpUSg",
            "otkxuT14nBiwzNJ3MPaOitOl9pVnXOaEHal_xkyNfKI"
        ],
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "exp": 1883000000,
        "verified_claims": {
            "verification": {
                "_sd": [
                    "7h4UE9qScvDKodXVCuoKfKBJpVBfXMF_TmAGVaZe3Sc",
                    "vTwe3raHIFYgFA3xaUD2aMxFz5oDo8iBu05qKlOg9Lw"
                ],
                "trust_framework": "de_aml",
                "evidence": [
                    {
                        "...": "tYJ0TDucyZZCRMbROG4qRO5vkPSFRxFhUELc18CSl3k"
                    },
                ]
            },
            "claims": {
                "_sd": [
                    "RiOiCn6_w5ZHaadkQMrcQJf0Jte5RwurRs54231DTlo",
                    "S_498bbpKzB6Eanftss0xc7cOaoneRr3pKr7NdRmsMo",
                    "WNA-UNK7F_zhsAb9syWO6IIQ1uHlTmOU8r8CvJ0cIMk",
                    "Wxh_sV3iRH9bgrTBJi-aYHNCLt-vjhX1sd-igOf_9lk",
                    "_O-wJiH3enSB4ROHntToQT8JmLtz-mhO2f1c89XoerQ",
                    "hvDXhwmGcJQsBCA2OtjuLAcwAMpDsaU0nkovcKOqWNE"
                ]
            }
        },
        "_sd_alg": "sha-256"
    }), true)]
    fn test_different_serialization_scenarios(#[case] original: serde_json::Value, #[case] is_valid: bool) {
        let deserialized = serde_json::from_value::<SdJwtClaims>(original.clone());

        assert_eq!(deserialized.is_ok(), is_valid);

        if is_valid {
            let serialized = serde_json::to_value(deserialized.unwrap()).unwrap();
            assert_eq!(serialized, original);
        }
    }

    #[test]
    fn sd_jwt_claims_features() {
        let value = json!({
            "_sd": [
                "CrQe7S5kqBAHt-nMYXgc6bdt2SH5aTY1sU_M-PgkjPI",
            ],
            "iss": "https://issuer.example.com/",
            "iat": 1683000000,
            "exp": 1883000000,
            "sub": "user_42",
            "object_with_digests": {
                "_sd": [
                    "gbOsI4Edq2x2Kw-w5wPEzakob9hV1cRD0ATN3oQL9JM",
                ],
                "field": "value",
            },
            "object_with_array_of_digests": {
                "array": [
                    {
                        "...": "pFndjkZ_VCzmyTa6UjlZo3dh-ko8aIKQc9DlGzhaVYo"
                    },
                ]
            },
            "array_of_digests": [
                {
                    "...": "pFndjkZ_VCzmyTa6UjlZo3dh-ko8aIKQc9DlGzhaVYo"
                },
            ],
            "array_of_object_with_digests": [
                {
                    "_sd": [
                        "jsu9yVulwQQlhFlM_3JlzMaSFzglhQG0DpfayQwLUK4"
                    ],
                },
                {
                    "...": "7Cf6JkPudry3lcbwHgeZ8khAv1U1OSlerP0VkBJrWZ0"
                }
            ],
            "_sd_alg": "sha-256",
            "cnf": {
                "jwk": {
                    "kty": "EC",
                    "crv": "P-256",
                    "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                    "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
                }
            }
        });
        let parsed: SdJwtClaims = serde_json::from_value(value).unwrap();

        let expected = SdJwtClaims {
            cnf: Some(RequiredKeyBinding::Jwk(Jwk {
                common: Default::default(),
                algorithm: AlgorithmParameters::EllipticCurve(EllipticCurveKeyParameters {
                    curve: EllipticCurve::P256,
                    key_type: EllipticCurveKeyType::EC,
                    x: "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc".to_string(),
                    y: "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ".to_string(),
                }),
            })),
            _sd_alg: Some(SdAlg::Sha256),
            vct_integrity: None,
            iss: "https://issuer.example.com/".parse::<HttpsUri>().unwrap(),
            iat: DateTimeSeconds::new(DateTime::from_timestamp(1683000000, 0).unwrap()),
            exp: DateTime::from_timestamp(1883000000, 0).map(DateTimeSeconds::new),
            nbf: None,
            vct: None,
            claims: ObjectClaims {
                _sd: Some(
                    vec!["CrQe7S5kqBAHt-nMYXgc6bdt2SH5aTY1sU_M-PgkjPI".to_string()]
                        .try_into()
                        .unwrap(),
                ),
                claims: HashMap::from([
                    ("sub".parse().unwrap(), ClaimValue::String("user_42".to_string())),
                    (
                        "object_with_digests".parse().unwrap(),
                        ClaimValue::Object(ObjectClaims {
                            _sd: Some(
                                vec!["gbOsI4Edq2x2Kw-w5wPEzakob9hV1cRD0ATN3oQL9JM".to_string()]
                                    .try_into()
                                    .unwrap(),
                            ),
                            claims: HashMap::from([(
                                "field".parse().unwrap(),
                                ClaimValue::String("value".to_string()),
                            )]),
                        }),
                    ),
                    (
                        "object_with_array_of_digests".parse().unwrap(),
                        ClaimValue::Object(ObjectClaims {
                            _sd: None,
                            claims: HashMap::from([(
                                "array".parse().unwrap(),
                                ClaimValue::Array(vec![ArrayClaim::Hash(
                                    "pFndjkZ_VCzmyTa6UjlZo3dh-ko8aIKQc9DlGzhaVYo".to_string(),
                                )]),
                            )]),
                        }),
                    ),
                    (
                        "array_of_digests".parse().unwrap(),
                        ClaimValue::Array(vec![ArrayClaim::Hash(
                            "pFndjkZ_VCzmyTa6UjlZo3dh-ko8aIKQc9DlGzhaVYo".to_string(),
                        )]),
                    ),
                    (
                        "array_of_object_with_digests".parse().unwrap(),
                        ClaimValue::Array(vec![
                            ArrayClaim::Value(ClaimValue::Object(ObjectClaims {
                                _sd: Some(
                                    vec!["jsu9yVulwQQlhFlM_3JlzMaSFzglhQG0DpfayQwLUK4".to_string()]
                                        .try_into()
                                        .unwrap(),
                                ),
                                claims: HashMap::new(),
                            })),
                            ArrayClaim::Hash("7Cf6JkPudry3lcbwHgeZ8khAv1U1OSlerP0VkBJrWZ0".to_string()),
                        ]),
                    ),
                ]),
            },
        };
        assert_eq!(parsed, expected);
    }

    fn prepare_disclosure(content: DisclosureContent) -> (String, Disclosure) {
        let disclosure = Disclosure::try_new(content).unwrap();
        let hasher = SdAlg::Sha256.hasher().unwrap();
        let digest = hasher.encoded_digest(disclosure.as_str());
        (digest, disclosure)
    }

    fn object_disclosure(key: &'static str, value: serde_json::Value) -> (String, Disclosure) {
        prepare_disclosure(DisclosureContent::ObjectProperty(
            crypto::utils::random_string(16),
            key.to_string(),
            value,
        ))
    }

    fn array_disclosure(value: serde_json::Value) -> (String, Disclosure) {
        prepare_disclosure(DisclosureContent::ArrayElement(crypto::utils::random_string(16), value))
    }

    fn parse_and_verify_disclosures(
        object: Value,
        disclosures: Vec<Disclosure>,
    ) -> Result<HashMap<String, Disclosure>> {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_keypair = ca.generate_issuer_mock().unwrap();
        let holder_privkey = SigningKey::random(&mut OsRng);

        let sd_jwt = SdJwtBuilder::new(json!({
            "iss": "https://issuer.example.com/",
            "iat": 1683000000,
        }))
        .unwrap()
        .finish(Integrity::from(""), &issuer_keypair, holder_privkey.verifying_key())
        .now_or_never()
        .unwrap()
        .unwrap()
        .to_string();

        let mut parsed = sd_jwt.parse::<UnverifiedSdJwt>().unwrap();
        parsed.disclosures = disclosures.into_iter().map(|d| d.to_string()).collect();

        UnverifiedSdJwt::parse_and_verify_disclosures(&parsed.disclosures, &serde_json::from_value(object).unwrap())
    }

    #[test]
    fn validate_disclosures() {
        let (root_object_claim_digest, root_object_claim) = object_disclosure("root_object_claim", json!("some_value"));
        let (nested_object_claim_digest, nested_object_claim) =
            object_disclosure("nested_object_claim", json!("some_value"));
        let (array_object_claim_digest, array_object_claim) =
            object_disclosure("array_object_claim", json!("some_value"));

        let (array_claim_digest, array_claim) = array_disclosure(json!("some_value"));
        let (nested_object_array_claim_digest, nested_object_array_claim) = array_disclosure(json!("some_value"));
        let (nested_array_claim_digest, nested_array_claim) = array_disclosure(json!("some_value"));

        let value = json!({
            "iss": "https://issuer.example.com/",
            "iat": 1683000000,
            "_sd": [
                &root_object_claim_digest,
            ],
            "static_claim": "static",
            "array_claim": [
                "static",
                { "...": &array_claim_digest }
            ],
            "nested_object_claim": {
                "_sd": [&nested_object_claim_digest],
                "array": [
                    "static",
                    { "...": &nested_object_array_claim_digest }
                ]
            },
            "nested_array_claim": [
                {
                    "_sd": [&array_object_claim_digest],
                    "array": [
                        { "...": &nested_array_claim_digest }
                    ]
                }
            ]
        });

        let disclosures = vec![
            root_object_claim,
            nested_object_claim,
            array_object_claim,
            array_claim,
            nested_object_array_claim,
            nested_array_claim,
        ];

        parse_and_verify_disclosures(value, disclosures).unwrap();
    }

    #[test]
    fn validate_digests_in_disclosures() {
        let (root_object_claim_digest, root_object_claim) = object_disclosure("root_object_claim", json!("some_value"));
        let (nested_object_claim_digest, nested_object_claim) =
            object_disclosure("nested_object_claim", json!("some_value"));
        let (array_object_claim_digest, array_object_claim) =
            object_disclosure("array_object_claim", json!("some_value"));

        let (array_claim_digest, array_claim) = array_disclosure(json!("some_value"));
        let (nested_object_array_claim_digest, nested_object_array_claim) = array_disclosure(json!("some_value"));
        let (nested_array_claim_digest, nested_array_claim) = array_disclosure(json!("some_value"));

        let (root_array_claim_digest, root_array_claim) = object_disclosure(
            "array_claim",
            json!([
                "static",
                { "...": &array_claim_digest }
            ]),
        );
        let (root_nested_object_claim_digest, root_nested_object_claim) = object_disclosure(
            "nested_object_claim",
            json!({
                "_sd": [&nested_object_claim_digest],
                "array": [
                    "static",
                    { "...": &nested_object_array_claim_digest }
                ]
            }),
        );
        let (root_nested_array_claim_digest, root_nested_array_claim) = object_disclosure(
            "nested_array_claim",
            json!([
                {
                    "_sd": [&array_object_claim_digest],
                    "array": [
                        { "...": &nested_array_claim_digest }
                    ]
                }
            ]),
        );

        let value = json!({
            "iss": "https://issuer.example.com/",
            "iat": 1683000000,
            "_sd": [
                &root_object_claim_digest,
                &root_array_claim_digest,
                &root_nested_object_claim_digest,
                &root_nested_array_claim_digest,
            ],
            "static_claim": "static"
        });

        let disclosures = vec![
            root_object_claim,
            nested_object_claim,
            array_object_claim,
            array_claim,
            nested_object_array_claim,
            nested_array_claim,
            root_array_claim,
            root_nested_array_claim,
            root_nested_object_claim,
        ];

        parse_and_verify_disclosures(value, disclosures).unwrap();
    }

    #[test]
    fn validate_disclosures_without_placeholder() {
        let (root_object_claim_digest, root_object_claim) = object_disclosure("root_object_claim", json!("some_value"));

        let value = json!({
            "iss": "https://issuer.example.com/",
            "iat": 1683000000,
            "static_claim": "static",
        });

        let disclosures = vec![root_object_claim];

        let error = parse_and_verify_disclosures(value, disclosures).unwrap_err();

        assert_matches!(error, Error::UnreferencedDisclosure(digest) if digest == root_object_claim_digest);
    }

    #[test]
    fn validate_disclosures_array_disclosure_in_object() {
        let (array_claim_digest, array_claim) = array_disclosure(json!("some_value"));

        let value = json!({
            "iss": "https://issuer.example.com/",
            "iat": 1683000000,
            "_sd": [
                &array_claim_digest,
            ],
        });

        let disclosures = vec![array_claim];

        let error = parse_and_verify_disclosures(value, disclosures).unwrap_err();

        assert_matches!(error, Error::DataTypeMismatch(message) if message == format!("Expected an Array element, but got an Object element for digest `{array_claim_digest}`"));
    }

    #[test]
    fn validate_disclosures_object_disclosure_in_array() {
        let (object_claim_digest, object_claim) = object_disclosure("some_field", json!("some_value"));

        let value = json!({
            "iss": "https://issuer.example.com/",
            "iat": 1683000000,
            "some_array": [
                { "...": &object_claim_digest }
            ]
        });

        let disclosures = vec![object_claim];

        let error = parse_and_verify_disclosures(value, disclosures).unwrap_err();

        assert_matches!(error, Error::DataTypeMismatch(message) if message == format!("Expected an Object element, but got an Array element for digest `{object_claim_digest}`"));
    }
}
