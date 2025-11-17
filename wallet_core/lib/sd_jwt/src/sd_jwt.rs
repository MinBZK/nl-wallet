use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::LazyLock;

use chrono::DateTime;
use chrono::Utc;
use derive_more::AsRef;
use indexmap::IndexMap;
use itertools::Itertools;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Validation;
use p256::ecdsa::VerifyingKey;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use serde_with::skip_serializing_none;
use ssri::Integrity;

use attestation_types::claim_path::ClaimPath;
use attestation_types::qualification::AttestationQualification;
use crypto::CredentialEcdsaKey;
use crypto::EcdsaKey;
use crypto::wscd::DisclosureWscd;
use crypto::wscd::WscdPoa;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateUsage;
use http_utils::urls::HttpsUri;
use jwt::EcdsaDecodingKey;
use jwt::JwtTyp;
use jwt::UnverifiedJwt;
use jwt::VerifiedJwt;
use jwt::error::JwkConversionError;
use jwt::headers::HeaderWithX5c;
use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;
use token_status_list::status_claim::StatusClaim;
use utils::date_time_seconds::DateTimeSeconds;
use utils::generator::Generator;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::claims::ClaimType;
use crate::claims::ClaimValue;
use crate::claims::ObjectClaims;
use crate::decoder::SdObjectDecoder;
use crate::disclosure::Disclosure;
use crate::error::ClaimError;
use crate::error::DecoderError;
use crate::error::SigningError;
use crate::hasher::Hasher;
use crate::key_binding_jwt::KbVerificationOptions;
use crate::key_binding_jwt::KeyBindingJwtBuilder;
use crate::key_binding_jwt::RequiredKeyBinding;
use crate::key_binding_jwt::SignedKeyBindingJwt;
use crate::key_binding_jwt::UnverifiedKeyBindingJwt;
use crate::key_binding_jwt::VerifiedKeyBindingJwt;
use crate::sd_alg::SdAlg;

/// SD-JWT payload trait used by this crate. Types implementing this trait provide accessors for the hash algorithm
/// (`_sd_alg`), holder binding (`cnf`) and the selectively disclosable claims tree (`claims`).
pub trait SdJwtClaims: JwtTyp {
    fn _sd_alg(&self) -> Option<SdAlg>;

    fn cnf(&self) -> &RequiredKeyBinding;

    fn claims(&self) -> &ClaimValue;
}

impl SdJwtClaims for SdJwtVcClaims {
    fn _sd_alg(&self) -> Option<SdAlg> {
        self._sd_alg
    }

    fn cnf(&self) -> &RequiredKeyBinding {
        &self.cnf
    }

    fn claims(&self) -> &ClaimValue {
        &self.claims
    }
}

/// An SD-JWT that has been split into parts but not verified yet.
///
/// There's no need to keep the SD-JWT as serialized form as there is no KB-JWT. Formats as `<Issuer-signed
/// JWT>~<Disclosure>~...~<Disclosure>~`.
///
/// Use [`UnverifiedSdJwt::into_verified_against_trust_anchors`] to validate the SD-JWT against provided trust
/// anchors.
#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct UnverifiedSdJwt<C = SdJwtVcClaims, H = HeaderWithX5c> {
    issuer_signed: UnverifiedJwt<C, H>,
    disclosures: Vec<String>,
}

impl<C, H> UnverifiedSdJwt<C, H> {
    pub(crate) fn new(issuer_signed: UnverifiedJwt<C, H>, disclosures: Vec<String>) -> Self {
        Self {
            issuer_signed,
            disclosures,
        }
    }

    fn dangerous_parse_unverified(&self) -> Result<VerifiedSdJwt<C, H>, DecoderError>
    where
        C: SdJwtClaims + DeserializeOwned,
        H: DeserializeOwned,
    {
        let issuer_signed = VerifiedJwt::<C, H>::dangerous_parse_unverified(self.issuer_signed.serialization())?;

        let hasher = issuer_signed.payload()._sd_alg().unwrap_or_default().hasher()?;
        let disclosures = self
            .disclosures
            .iter()
            .map(|segment| {
                let disclosure: Disclosure = segment.parse()?;
                let key = hasher.encoded_digest(disclosure.encoded());
                Result::Ok((key, disclosure))
            })
            .collect::<Result<_, DecoderError>>()?;

        Ok(VerifiedSdJwt {
            issuer_signed,
            disclosures,
        })
    }
}

impl<C, H> Display for UnverifiedSdJwt<C, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &std::iter::once(self.issuer_signed.serialization())
                .chain(self.disclosures.iter().map(String::as_str))
                .map(|s| format!("{s}~"))
                .collect::<String>(),
        )
    }
}

impl<C, H> FromStr for UnverifiedSdJwt<C, H> {
    type Err = DecoderError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.strip_suffix("~").ok_or(DecoderError::MissingFinalTilde)?;

        let mut segments = s.split('~');
        let issuer_signed = segments.next().ok_or(DecoderError::MissingIssuerSignedJwt)?.parse()?;
        let disclosures = segments.map(ToString::to_string).collect_vec();

        Result::Ok(UnverifiedSdJwt {
            issuer_signed,
            disclosures,
        })
    }
}

impl UnverifiedSdJwt {
    /// Verifies the issuer-signed part (using `x5c` against `trust_anchors`) and parses and verifies disclosures,
    /// producing a [`VerifiedSdJwt`].
    pub fn into_verified_against_trust_anchors(
        self,
        trust_anchors: &[TrustAnchor],
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<VerifiedSdJwt, DecoderError> {
        let issuer_signed = self.issuer_signed.into_verified_against_trust_anchors(
            &SD_JWT_VALIDATIONS,
            time,
            CertificateUsage::Mdl,
            trust_anchors,
        )?;

        let disclosures = Self::parse_and_verify_disclosures(&self.disclosures, issuer_signed.payload())?;

        Ok(VerifiedSdJwt {
            issuer_signed,
            disclosures,
        })
    }
}

impl<C: SdJwtClaims, H> UnverifiedSdJwt<C, H> {
    /// Parses and verifies disclosures according to <https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-22.html#section-7.1>
    fn parse_and_verify_disclosures(
        disclosures: &[String],
        sd_jwt_claims: &C,
    ) -> Result<IndexMap<String, Disclosure>, DecoderError> {
        let hasher = sd_jwt_claims._sd_alg().unwrap_or_default().hasher()?;
        let mut placeholder_digests: IndexMap<String, ClaimType> =
            sd_jwt_claims.claims().digests().into_iter().collect();

        let disclosures: IndexMap<String, Disclosure> = disclosures
            .iter()
            .map(|disclosure| {
                let digest = hasher.encoded_digest(disclosure);
                let disclosure = disclosure.parse::<Disclosure>()?;

                for (digest, claim_type) in disclosure.digests() {
                    // 7.1.4. If any digest value is encountered more than once in the Issuer-signed JWT payload
                    // (directly or recursively via other Disclosures), the SD-JWT MUST be rejected.
                    if placeholder_digests.insert(digest.clone(), claim_type).is_some() {
                        return Err(DecoderError::DuplicateHash(digest));
                    }
                }

                Result::Ok((digest, disclosure))
            })
            .try_collect()?;

        // 7.1.5. If any Disclosure was not referenced by digest value in the Issuer-signed JWT (directly or recursively
        // via other Disclosures), the SD-JWT MUST be rejected.
        disclosures
            .iter()
            .try_for_each(|(digest, disclosure)| match placeholder_digests.get(digest) {
                // For any disclosure that is referenced, verify that the hash type matches the digest hash type.
                Some(digest_claim_type) => Ok(disclosure.verify_matching_claim_type(*digest_claim_type, digest)?),
                None => Err(DecoderError::UnreferencedDisclosure(digest.clone())),
            })?;

        Ok(disclosures)
    }
}

impl From<UnsignedSdJwtPresentation> for UnverifiedSdJwt {
    fn from(presentation: UnsignedSdJwtPresentation) -> Self {
        presentation.0.into()
    }
}

impl From<VerifiedSdJwt> for UnverifiedSdJwt {
    fn from(sd_jwt: VerifiedSdJwt) -> Self {
        let disclosures = sd_jwt
            .disclosures
            .into_values()
            .map(|disclosure| disclosure.encoded)
            .collect();

        Self {
            issuer_signed: sd_jwt.issuer_signed.into(),
            disclosures,
        }
    }
}

/// SD-JWT VC claims type used by the builder and verifier.
///
/// Holds VC metadata (`vct`, `iss`), validity information (`iat` and optionally `exp` and `nbf`), the holder binding
/// (`cnf`), and the selectively-disclosable claims tree in `claims`.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-22.html#name-issuer-signed-jwt>
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SdJwtVcClaims {
    pub _sd_alg: Option<SdAlg>,

    pub cnf: RequiredKeyBinding,

    // Even though we want this to be mandatory, we allow it to be optional in order for the examples from the spec
    // to parse.
    #[serde(rename = "vct#integrity")]
    pub vct_integrity: Option<Integrity>,

    pub vct: String,

    pub iss: HttpsUri,

    pub iat: DateTimeSeconds,

    pub exp: Option<DateTimeSeconds>,

    pub nbf: Option<DateTimeSeconds>,

    // Even though we want this to be mandatory, we allow it to be optional in order for the examples from the spec
    // to parse.
    pub status: Option<StatusClaim>,

    // Even though we want this to be mandatory, we allow it to be optional in order for the examples from the spec
    // to parse.
    pub attestation_qualification: Option<AttestationQualification>,

    // In practice this should always be a `ClaimValue::Object`, however `ClaimValue` is used here instead of
    // `ObjectClaims` to make is possible to call `ClaimValue::traverse_by_claim_paths` at this level and return `self`
    // for an empty `claim_path`.
    #[serde(flatten)]
    pub claims: ClaimValue,
}

impl SdJwtVcClaims {
    pub fn claims(&self) -> &ObjectClaims {
        let ClaimValue::Object(claims) = &self.claims else {
            panic!("Should always be an Object")
        };
        claims
    }
}

/// Verified SD-JWT consisting of a verified issuer-signed JWT and verified disclosures, referenced by their digest
/// values.
///
/// Formats as `<Issuer-signed JWT>~<Disclosure>~...~<Disclosure>~`.
#[derive(Debug, Clone, Eq, PartialEq, SerializeDisplay)]
pub struct VerifiedSdJwt<C = SdJwtVcClaims, H = HeaderWithX5c> {
    issuer_signed: VerifiedJwt<C, H>,

    disclosures: IndexMap<String, Disclosure>,
}

impl<C, H> Display for VerifiedSdJwt<C, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &std::iter::once(self.issuer_signed.jwt().serialization())
                .chain(self.disclosures.values().map(|d| d.encoded()))
                .map(|s| format!("{s}~"))
                .collect::<String>(),
        )
    }
}

impl<C, H> VerifiedSdJwt<C, H> {
    pub(crate) fn dangerous_new(issuer_signed: VerifiedJwt<C, H>, disclosures: IndexMap<String, Disclosure>) -> Self {
        Self {
            issuer_signed,
            disclosures,
        }
    }

    pub fn claims(&self) -> &C {
        self.issuer_signed.payload()
    }

    pub fn into_claims(self) -> C {
        self.issuer_signed.into_payload()
    }

    pub fn disclosures(&self) -> &IndexMap<String, Disclosure> {
        &self.disclosures
    }
}

impl<C> VerifiedSdJwt<C, HeaderWithX5c> {
    pub fn issuer_certificate_chain(&self) -> &VecNonEmpty<BorrowingCertificate> {
        &self.issuer_signed.header().x5c
    }

    pub fn issuer_certificate(&self) -> &BorrowingCertificate {
        self.issuer_signed.header().x5c.first()
    }
}

impl VerifiedSdJwt {
    /// Prepares this SD-JWT for presentation, returning a builder that can be used to select which claims to disclose.
    pub fn into_presentation_builder(self) -> SdJwtPresentationBuilder {
        SdJwtPresentationBuilder::new(self)
    }

    pub fn holder_pubkey(&self) -> Result<VerifyingKey, JwkConversionError> {
        self.claims().cnf().verifying_key()
    }

    /// Parses an SD-JWT into its components as [`VerifiedSdJwt`] without verifying the signature. Note that this should
    /// only be used when receiving the SD-JWT over a trusted channel (i.e. from the database).
    pub fn dangerous_parse_unverified(s: &str) -> Result<Self, DecoderError> {
        let serialization = s.parse::<UnverifiedSdJwt>()?;
        serialization.dangerous_parse_unverified()
    }

    pub fn verify_selective_disclosability(
        &self,
        claim_path: &[ClaimPath],
        sd_metadata: &HashMap<Vec<ClaimPath>, ClaimSelectiveDisclosureMetadata>,
    ) -> Result<(), ClaimError> {
        self.claims()
            .claims
            .verify_selective_disclosability(claim_path, 0, &self.disclosures, sd_metadata)
    }
}

#[inline]
pub fn verify_selective_disclosability(
    should_be_disclosable: &ClaimSelectiveDisclosureMetadata,
    is_actually_disclosable: bool,
    claim_path: &[ClaimPath],
) -> Result<(), ClaimError> {
    match (should_be_disclosable, is_actually_disclosable) {
        (ClaimSelectiveDisclosureMetadata::Always, false) | (ClaimSelectiveDisclosureMetadata::Never, true) => {
            Err(ClaimError::SelectiveDisclosabilityMismatch(
                claim_path.to_vec(),
                *should_be_disclosable,
                is_actually_disclosable,
            ))
        }
        _ => Ok(()),
    }
}

impl<H> VerifiedSdJwt<SdJwtVcClaims, H> {
    /// Decodes the SD-protected claims by substituting matched disclosures, returning a plain `ObjectClaims` structure
    /// without `_sd` claims or digests.
    pub fn decoded_claims(&self) -> Result<ObjectClaims, DecoderError> {
        let claims = SdObjectDecoder::decode(self.claims(), &self.disclosures)?;
        let ClaimValue::Object(claims) = claims.claims else {
            panic!("should always be an Object after SdObjectDecoder::decode")
        };

        Ok(claims)
    }
}

impl UnsignedSdJwtPresentation {
    /// Create multiple `SdJwtPresentation`s by having the WSCD sign multiple `UnsignedSdJwtPresentation`s,
    /// using the contents of a single `KeyBindingJwtBuilder`.
    pub async fn sign_multiple<I, W, K, P>(
        unsigned_presentations_and_keys_ids: VecNonEmpty<(UnsignedSdJwtPresentation, I)>,
        key_binding_jwt_builder: KeyBindingJwtBuilder,
        wscd: &W,
        poa_input: P::Input,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<(VecNonEmpty<SignedSdJwtPresentation>, Option<P>), SigningError>
    where
        I: Into<String>,
        W: DisclosureWscd<Key = K, Poa = P>,
        K: CredentialEcdsaKey,
        P: WscdPoa,
    {
        // Create the WSCD keys from the provided key identifiers and public keys present in the `cnf` claim.
        // Note that the latter is not actually used, as all we do is signing.
        let sd_jwts_and_keys: VecNonEmpty<(VerifiedSdJwt, _)> = unsigned_presentations_and_keys_ids
            .into_nonempty_iter()
            .map(|(UnsignedSdJwtPresentation(sd_jwt), key_identifier)| {
                let key = wscd.new_key(key_identifier, sd_jwt.holder_pubkey()?);

                Ok((sd_jwt, key))
            })
            .collect::<Result<_, JwkConversionError>>()?;

        // Have the WSCD create `KeyBindingJwt`s and the PoA, if required.
        let (key_binding_jwts, poa) = key_binding_jwt_builder
            .finish_multiple(&sd_jwts_and_keys, wscd, poa_input, time)
            .await?;

        // Combine the `SdJwt`s with the `KeyBindingJwt`s to create `SdJwtPresentation`s.
        let sd_jwt_presentations = sd_jwts_and_keys
            .into_nonempty_iter()
            .zip(key_binding_jwts)
            .map(|((sd_jwt, _), key_binding_jwt)| SignedSdJwtPresentation {
                sd_jwt,
                key_binding_jwt,
            })
            .collect();

        Ok((sd_jwt_presentations, poa))
    }
}

/// Parsed but not yet verified SD-JWT Presentation consisting of an SD-JWT and a Key Binding JWT (KB-JWT).
///
/// Formats as `<SD-JWT>~<KB-JWT>`.
#[derive(Debug, Clone, Eq, PartialEq, SerializeDisplay, DeserializeFromStr)]
pub struct UnverifiedSdJwtPresentation<C = SdJwtVcClaims, H = HeaderWithX5c> {
    sd_jwt: UnverifiedSdJwt<C, H>,
    key_binding_jwt: UnverifiedKeyBindingJwt,
}

impl<C, H> FromStr for UnverifiedSdJwtPresentation<C, H> {
    type Err = DecoderError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let sd_jwt_end = s.rfind("~").ok_or(DecoderError::MissingSegments)? + 1;
        // the SD-JWT part includes the trailing '~'

        let sd_jwt = s[..sd_jwt_end].parse::<UnverifiedSdJwt<C, H>>()?;
        let key_binding_jwt = s[sd_jwt_end..].parse::<UnverifiedKeyBindingJwt>()?;

        Ok(Self {
            sd_jwt,
            key_binding_jwt,
        })
    }
}

impl<C, H> Display for UnverifiedSdJwtPresentation<C, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.sd_jwt, self.key_binding_jwt)
    }
}

impl UnverifiedSdJwtPresentation {
    /// Parses an SD-JWT into its components as [`VerifiedSdJwtPresentation`] while verifying against a set of trust
    /// anchors.
    ///
    /// Verifies the presentation by:
    /// 1) validating the issuer-signed JWT against `trust_anchors`,
    /// 2) validating the KB-JWT against the public key from the `cnf` claim in the verified issuer-signed JWT,
    /// 3) parsing/verifying disclosures.
    pub fn into_verified_against_trust_anchors(
        self,
        trust_anchors: &[TrustAnchor],
        kb_verification_options: &KbVerificationOptions,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<VerifiedSdJwtPresentation, DecoderError> {
        // we first verify the SD-JWT, then extract the JWK from the `cnf` claim and use that to verify the KB-JWT
        // before parsing the disclosures
        let issuer_signed = self.sd_jwt.issuer_signed.into_verified_against_trust_anchors(
            &SD_JWT_VALIDATIONS,
            time,
            CertificateUsage::Mdl,
            trust_anchors,
        )?;

        let key_binding_jwt = self.key_binding_jwt.into_verified(
            &EcdsaDecodingKey::from(&issuer_signed.payload().cnf().verifying_key()?),
            kb_verification_options,
            time,
        )?;

        let disclosures = UnverifiedSdJwt::<SdJwtVcClaims, HeaderWithX5c>::parse_and_verify_disclosures(
            &self.sd_jwt.disclosures,
            issuer_signed.payload(),
        )?;
        Ok(VerifiedSdJwtPresentation {
            sd_jwt: VerifiedSdJwt {
                issuer_signed,
                disclosures,
            },
            key_binding_jwt,
        })
    }
}

/// Verified SD-JWT Presentation combining a verified SD-JWT and a verified KB-JWT.
#[derive(Debug, Clone, Eq, PartialEq, SerializeDisplay)]
pub struct VerifiedSdJwtPresentation<C = SdJwtVcClaims, H = HeaderWithX5c> {
    sd_jwt: VerifiedSdJwt<C, H>,
    key_binding_jwt: VerifiedKeyBindingJwt,
}

impl<C, H> Display for VerifiedSdJwtPresentation<C, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.sd_jwt, self.key_binding_jwt)
    }
}

impl<C, H> VerifiedSdJwtPresentation<C, H> {
    pub fn sd_jwt(&self) -> &VerifiedSdJwt<C, H> {
        &self.sd_jwt
    }

    pub fn into_claims(self) -> C {
        self.sd_jwt.into_claims()
    }
}

impl<C> VerifiedSdJwtPresentation<C, HeaderWithX5c> {
    pub fn issuer_certificate(&self) -> &BorrowingCertificate {
        self.sd_jwt.issuer_certificate()
    }
}

/// Builder to construct an SD-JWT presentation by selecting which claims to disclose.
///
/// Call [`SdJwtPresentationBuilder::disclose`] for each path, then [`SdJwtPresentationBuilder::finish`] to get an
/// [`UnsignedSdJwtPresentation`].
#[derive(Clone)]
pub struct SdJwtPresentationBuilder {
    sd_jwt: VerifiedSdJwt,

    /// Non-disclosed attributes. All attributes start here. Calling `disclose()` moves an attribute from here
    /// to `disclosed`.
    nondisclosed: IndexMap<String, Disclosure>,

    /// Digests to be disclosed.
    digests_to_be_disclosed: HashSet<String>,

    /// A helper object containing both non-selectively disclosable JWT claims and the `_sd` digests,
    /// used by `digests_to_disclose()`.
    full_payload: SdJwtVcClaims,
}

/// An SD-JWT ready for presentation after selecting disclosures, before binding with a KB-JWT.
#[derive(Debug, Clone, Eq, PartialEq, AsRef, Serialize)]
pub struct UnsignedSdJwtPresentation(VerifiedSdJwt);

impl SdJwtPresentationBuilder {
    fn new(mut sd_jwt: VerifiedSdJwt) -> Self {
        let full_payload = sd_jwt.issuer_signed.payload().to_owned();
        let nondisclosed = std::mem::take(&mut sd_jwt.disclosures);
        Self {
            sd_jwt,
            nondisclosed,
            digests_to_be_disclosed: HashSet::new(),
            full_payload,
        }
    }

    /// Select a path to disclose in the presentation. Should be called for each path to disclose.
    pub fn disclose(mut self, path: &VecNonEmpty<ClaimPath>) -> Result<Self, ClaimError> {
        // Gather all digests to be disclosed into a set. This can include intermediary attributes as well
        self.digests_to_be_disclosed.extend({
            let mut path_segments = path.as_ref().iter().peekable();
            self.full_payload
                .claims
                .digests_to_disclose(&mut path_segments, &self.nondisclosed, false)?
                .into_iter()
                .map(String::from)
        });

        Ok(self)
    }

    /// Returns an [`UnsignedSdJwtPresentation`] containing the original issuer signed JWT and a list of disclosures to
    /// be disclose. Can be turned into a [`SignedSdJwtPresentation`] using [`UnsignedSdJwtPresentation::sign`].
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
            .fold(IndexMap::new(), |mut disclosures, digest| {
                let disclosure = nondisclosed
                    .shift_remove(&digest)
                    .expect("disclosure should be present");
                disclosures.insert(digest, disclosure);
                disclosures
            });

        UnsignedSdJwtPresentation(sd_jwt)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, SerializeDisplay)]
pub struct SignedSdJwtPresentation {
    sd_jwt: VerifiedSdJwt<SdJwtVcClaims, HeaderWithX5c>,
    key_binding_jwt: SignedKeyBindingJwt,
}

impl Display for SignedSdJwtPresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.sd_jwt, self.key_binding_jwt)
    }
}

impl SignedSdJwtPresentation {
    pub fn into_unverified(self) -> UnverifiedSdJwtPresentation {
        self.into()
    }

    pub fn key_binding_jwt(&self) -> &SignedKeyBindingJwt {
        &self.key_binding_jwt
    }
}

impl From<SignedSdJwtPresentation> for UnverifiedSdJwtPresentation {
    fn from(value: SignedSdJwtPresentation) -> Self {
        Self {
            sd_jwt: value.sd_jwt.into(),
            key_binding_jwt: UnverifiedKeyBindingJwt::from_signed(value.key_binding_jwt),
        }
    }
}

impl UnsignedSdJwtPresentation {
    /// Signs the underlying [`VerifiedSdJwt`] and returns an SD-JWT presentation containing the (verified) issuer
    /// signed SD-JWT and signed KB-JWT.
    pub async fn sign(
        self,
        key_binding_jwt_builder: KeyBindingJwtBuilder,
        signing_key: &impl EcdsaKey,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<SignedSdJwtPresentation, SigningError> {
        let sd_jwt = self.0;

        let key_binding_jwt = key_binding_jwt_builder.finish(&sd_jwt, signing_key, time).await?;
        let sd_jwt_presentation = SignedSdJwtPresentation {
            sd_jwt,
            key_binding_jwt,
        };

        Ok(sd_jwt_presentation)
    }
}

static SD_JWT_VALIDATIONS: LazyLock<Validation> = LazyLock::new(|| {
    let mut validation = Validation::new(Algorithm::ES256);
    validation.validate_aud = false;
    validation.validate_nbf = true;
    validation.leeway = 0;
    validation.required_spec_claims.clear(); // remove "exp" from required claims
    validation
});

#[cfg(any(test, feature = "examples"))]
impl<C, H, E> UnverifiedSdJwt<C, H>
where
    C: SdJwtClaims + DeserializeOwned + JwtTyp,
    H: TryFrom<jwt::Header, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    pub(crate) fn into_verified(self, pubkey: &EcdsaDecodingKey) -> Result<VerifiedSdJwt<C, H>, DecoderError> {
        let issuer_signed = self.issuer_signed.into_verified(pubkey, &SD_JWT_VALIDATIONS)?;
        let disclosures = Self::parse_and_verify_disclosures(&self.disclosures, issuer_signed.payload())?;
        Ok(VerifiedSdJwt {
            issuer_signed,
            disclosures,
        })
    }
}

#[cfg(any(test, feature = "examples"))]
impl<C, H, E> UnverifiedSdJwtPresentation<C, H>
where
    C: SdJwtClaims + DeserializeOwned + JwtTyp,
    H: TryFrom<jwt::Header, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    pub(crate) fn into_verified(
        self,
        issuer_pubkey: &EcdsaDecodingKey,
        kb_expected_aud: &str,
        kb_expected_nonce: &str,
        kb_iat_acceptance_window: std::time::Duration,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<VerifiedSdJwtPresentation<C, H>, DecoderError> {
        // we first verify the SD-JWT, then extract the JWK from the `cnf` claim and use that to verify the KB-JWT
        // before parsing the disclosures

        use std::time::Duration;
        let issuer_signed = self
            .sd_jwt
            .issuer_signed
            .into_verified(issuer_pubkey, &SD_JWT_VALIDATIONS)?;

        let kb_verification_options = KbVerificationOptions {
            expected_aud: kb_expected_aud,
            expected_nonce: kb_expected_nonce,
            iat_leeway: Duration::ZERO,
            iat_acceptance_window: kb_iat_acceptance_window,
        };
        let key_binding_jwt = self.key_binding_jwt.into_verified(
            &EcdsaDecodingKey::from(&issuer_signed.payload().cnf().verifying_key()?),
            &kb_verification_options,
            time,
        )?;

        let disclosures =
            UnverifiedSdJwt::<C, H>::parse_and_verify_disclosures(&self.sd_jwt.disclosures, issuer_signed.payload())?;
        Ok(VerifiedSdJwtPresentation {
            sd_jwt: VerifiedSdJwt {
                issuer_signed,
                disclosures,
            },
            key_binding_jwt,
        })
    }
}

#[cfg(any(test, feature = "examples"))]
mod examples {
    use std::time::Duration;

    use chrono::DateTime;
    use chrono::Utc;
    use p256::ecdsa::VerifyingKey;
    use serde_json::Value;
    use serde_json::json;

    use attestation_types::qualification::AttestationQualification;
    use jwt::jwk::jwk_from_p256;
    use token_status_list::status_claim::StatusClaim;
    use utils::generator::Generator;

    use crate::key_binding_jwt::RequiredKeyBinding;

    use super::SdJwtVcClaims;

    impl SdJwtVcClaims {
        pub fn pid_example(holder_pubkey: &VerifyingKey, time: &impl Generator<DateTime<Utc>>) -> Self {
            SdJwtVcClaims {
                _sd_alg: None,
                cnf: RequiredKeyBinding::Jwk(jwk_from_p256(holder_pubkey).unwrap()),
                vct_integrity: Some("sha256-47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=".parse().unwrap()),
                vct: "urn:eudi:pid:nl:1".to_owned(),
                iss: "https://cert.issuer.example.com".parse().unwrap(),
                iat: time.generate().into(),
                exp: Some((time.generate() + Duration::from_secs(365 * 24 * 60 * 60)).into()),
                nbf: None,
                attestation_qualification: Some(AttestationQualification::QEAA),
                status: Some(StatusClaim::new_mock()),
                claims: serde_json::from_value(json!({
                    "bsn": "999999999",
                    "recovery_code": "cff292503cba8c4fbf2e5820dcdc468ae00f40c87b1af35513375800128fc00d",
                    "given_name": "Willeke Liselotte",
                    "family_name": "De Bruijn",
                    "birthdate": "1940-01-01"
                }))
                .unwrap(),
            }
        }

        pub fn example_from_json(
            holder_public_key: &VerifyingKey,
            claims: Value,
            time: &impl Generator<DateTime<Utc>>,
        ) -> Self {
            SdJwtVcClaims {
                _sd_alg: None,
                cnf: RequiredKeyBinding::Jwk(jwk_from_p256(holder_public_key).unwrap()),
                vct_integrity: Some("sha256-47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=".parse().unwrap()),
                vct: "urn:eudi:pid:nl:1".to_owned(),
                iss: "https://cert.issuer.example.com".parse().unwrap(),
                iat: time.generate().into(),
                exp: None,
                nbf: None,
                attestation_qualification: Some(AttestationQualification::PubEAA),
                status: Some(StatusClaim::new_mock()),
                claims: serde_json::from_value(claims).unwrap(),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::time::Duration;

    use assert_matches::assert_matches;
    use chrono::DateTime;
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

    use crypto::server_keys::generate::Ca;
    use http_utils::urls::HttpsUri;
    use jwt::Header;
    use jwt::error::JwtError;
    use utils::date_time_seconds::DateTimeSeconds;
    use utils::generator::mock::MockTimeGenerator;

    use crate::builder::SdJwtBuilder;
    use crate::claims::ArrayClaim;
    use crate::disclosure::Disclosure;
    use crate::disclosure::DisclosureContent;
    use crate::examples::KeyBindingExampleTimeGenerator;
    use crate::examples::*;
    use crate::key_binding_jwt::KeyBindingJwtBuilder;
    use crate::key_binding_jwt::RequiredKeyBinding;
    use crate::sd_alg::SdAlg;
    use crate::sd_jwt::ClaimValue;
    use crate::sd_jwt::ObjectClaims;
    use crate::sd_jwt::SdJwtVcClaims;
    use crate::sd_jwt::UnverifiedSdJwt;
    use crate::test::ARRAY_DIGEST_KEY;
    use crate::test::DIGESTS_KEY;
    use crate::test::array_disclosure;
    use crate::test::object_disclosure;

    use super::*;

    #[rstest]
    #[case(SIMPLE_STRUCTURED_SD_JWT, 2)]
    #[case(COMPLEX_STRUCTURED_SD_JWT, 6)]
    fn parse_sd_jwt(#[case] encoded: &str, #[case] expected_disclosures: usize) {
        let sd_jwt = encoded
            .parse::<UnverifiedSdJwt<SdJwtExampleClaims, Header>>()
            .unwrap()
            .into_verified(&examples_sd_jwt_decoding_key())
            .unwrap();
        assert_eq!(sd_jwt.disclosures.len(), expected_disclosures);
    }

    #[test]
    fn parse_vc() {
        let sd_jwt = SD_JWT_VC
            .parse::<UnverifiedSdJwt<SdJwtVcClaims, Header>>()
            .unwrap()
            .into_verified(&examples_sd_jwt_decoding_key())
            .unwrap();
        assert_eq!(sd_jwt.disclosures.len(), 21);
    }

    #[test]
    fn parse_kb() {
        WITH_KB_SD_JWT
            .parse::<UnverifiedSdJwtPresentation<SdJwtExampleClaims, Header>>()
            .unwrap(); // we can only verify SD JWTs with a `cnf` field
    }

    #[test]
    fn parse_vc_with_kb() {
        SD_JWT_VC_WITH_KB
            .parse::<UnverifiedSdJwtPresentation<SdJwtVcClaims, Header>>()
            .unwrap()
            .into_verified(
                &examples_sd_jwt_decoding_key(),
                WITH_KB_SD_JWT_AUD,
                WITH_KB_SD_JWT_NONCE,
                Duration::from_secs(10 * 60),
                &KeyBindingExampleTimeGenerator,
            )
            .unwrap();
    }

    #[tokio::test]
    async fn test_parse_should_error_for_expired_jwt() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_keypair = ca.generate_issuer_mock().unwrap();

        let holder_key = SigningKey::random(&mut OsRng);
        let sd_jwt = SdJwtBuilder::new(SdJwtVcClaims::pid_example(
            holder_key.verifying_key(),
            &MockTimeGenerator::epoch(),
        ))
        .finish(&issuer_keypair)
        .await
        .unwrap()
        .to_string();

        let err = sd_jwt
            .parse::<UnverifiedSdJwt<SdJwtVcClaims, Header>>()
            .unwrap()
            .into_verified(&EcdsaDecodingKey::from(issuer_keypair.certificate().public_key()))
            .expect_err("should fail");

        assert_matches!(err, DecoderError::JwtParsing(JwtError::Validation(err)) if err.kind() == &ErrorKind::ExpiredSignature);
    }

    #[test]
    fn round_trip_ser_des() {
        let sd_jwt = SIMPLE_STRUCTURED_SD_JWT
            .parse::<UnverifiedSdJwt<SdJwtExampleClaims, Header>>()
            .unwrap()
            .into_verified(&examples_sd_jwt_decoding_key())
            .unwrap();

        let VerifiedSdJwt {
            issuer_signed: expected_jwt,
            disclosures: expected_disclosures,
        } = SIMPLE_STRUCTURED_SD_JWT
            .parse::<UnverifiedSdJwt<SdJwtExampleClaims, Header>>()
            .unwrap()
            .dangerous_parse_unverified()
            .unwrap();

        assert_eq!(sd_jwt.disclosures(), &expected_disclosures);
        assert_eq!(sd_jwt.issuer_signed.payload(), expected_jwt.payload());

        let sd_jwt_str = sd_jwt.to_string();
        assert_eq!(sd_jwt_str, SIMPLE_STRUCTURED_SD_JWT);
    }

    #[test]
    fn parse_invalid_disclosure() {
        let result = INVALID_DISCLOSURE_SD_JWT
            .parse::<UnverifiedSdJwt<SdJwtExampleClaims, Header>>()
            .unwrap()
            .into_verified(&examples_sd_jwt_decoding_key());

        assert_matches!(result, Err(DecoderError::JsonDeserialization(_)));
    }

    fn create_presentation(
        object: serde_json::Value,
        conceal_paths: &[Vec<&str>],
        disclose_paths: &[Vec<&str>],
    ) -> SignedSdJwtPresentation {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_keypair = ca.generate_issuer_mock().unwrap();
        let holder_key = SigningKey::random(&mut OsRng);

        let signed_sd_jwt = conceal_paths
            .iter()
            .fold(
                SdJwtBuilder::new(SdJwtVcClaims::example_from_json(
                    holder_key.verifying_key(),
                    object,
                    &MockTimeGenerator::default(),
                )),
                |builder, path| {
                    builder
                        .make_concealable(
                            path.iter()
                                .map(|p| p.parse().unwrap())
                                .collect_vec()
                                .try_into()
                                .unwrap(),
                        )
                        .unwrap()
                },
            )
            .finish(&issuer_keypair)
            .now_or_never()
            .unwrap()
            .unwrap();

        // the signed SD-JWT must be verified before it can be used to create a presentation
        let verified_sd_jwt = signed_sd_jwt.into_verified();
        disclose_paths
            .iter()
            .fold(verified_sd_jwt.into_presentation_builder(), |builder, path| {
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
                KeyBindingJwtBuilder::new("aud".to_string(), "nonce".to_string()),
                &holder_key,
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap()
            .unwrap()
    }

    #[rstest]
    #[case::default_nothing_disclosed(
        json!({
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
            "address": {"street": "Main st.", "house_number": 4 }
        }),
        &[vec!["address"]],
        &[vec!["address"]],
        &["address"],
        &[],
    )]
    #[case::array(
        json!({
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

        let claims = presentation.sd_jwt.issuer_signed.payload();
        let serde_json::Value::Object(properties) = serde_json::to_value(&claims.claims).unwrap() else {
            panic!("unexpected")
        };
        let not_selectively_disclosable_paths = get_paths(&properties);

        assert_eq!(
            HashSet::from_iter(expected_disclosure_paths.iter().map(|path| path.parse().unwrap())),
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
            "nationalities": ["NL", "DE"]
        }),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"]],
        &[vec!["nationalities", "null"]],
        &["NL", "DE"],
        &["/nationalities"],
    )]
    #[case::array_all_non_sd(
        json!({
            "nationalities": ["NL", "DE"]
        }),
        &[vec!["nationalities", "1"]],
        &[vec!["nationalities", "null"]],
        &["DE"],
        &["/nationalities/NL", "/nationalities"],
    )]
    #[case::array(
        json!({
            "nationalities": ["NL", "DE"]
        }),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"], vec!["nationalities"]],
        &[vec!["nationalities", "null"]],
        &["nationalities", "NL", "DE"],
        &[],
    )]
    #[case::array(
        json!({
            "nationalities": ["NL", "DE"]
        }),
        &[vec!["nationalities", "0"]],
        &[vec!["nationalities", "0"]],
        &["NL"],
        &["/nationalities/DE", "/nationalities"],
    )]
    #[case::array_index_non_sd(
        json!({
            "nationalities": ["NL", "DE"]
        }),
        &[vec!["nationalities", "0"]],
        &[vec!["nationalities", "0"], vec!["nationalities", "1"]],
        &["NL"],
        &["/nationalities/DE", "/nationalities"],
    )]
    #[case::array(
        json!({
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
            "nationalities": [{"country": "NL"}, {"country": "DE"}]
        }),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"], vec!["nationalities"]],
        &[vec!["nationalities", "null"]],
        &["nationalities", "country"],
        &[],
    )]
    #[case::array(
        json!({
            "nationalities": ["NL", "DE"]
        }),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"]],
        &[vec!["nationalities", "1"]],
        &["DE"],
        &["/nationalities"],
    )]
    #[case::array(
        json!({
            "nationalities": ["NL", "DE"]
        }),
        &[vec!["nationalities", "0"], vec!["nationalities", "1"], vec!["nationalities"]],
        &[vec!["nationalities", "1"]],
        &["nationalities", "DE"],
        &[],
    )]
    #[case::array(
        json!({
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

        let payload = presentation.sd_jwt.issuer_signed.payload();
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
                DisclosureContent::ArrayElement(_, array_claim) => match array_claim {
                    ArrayClaim::Value(value) => match value {
                        ClaimValue::Object(object_claims) => {
                            for (key, _value) in object_claims.claims {
                                actual_disclosed_paths_or_values.insert(key);
                            }
                        }
                        ClaimValue::String(value) => {
                            actual_disclosed_paths_or_values.insert(value.parse().unwrap());
                        }
                        _ => {}
                    },
                    ArrayClaim::Hash { .. } => {}
                },
            }
        }

        assert_eq!(
            HashSet::from_iter(
                expected_disclosure_paths_or_values
                    .iter()
                    .map(|path| path.parse().unwrap())
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
        "vct": "com:example:pid:1",
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "given_name": "Alice",
        "_sd": ["X9yH0Ajrdm1Oij4tWso9UzzKJvPoDxwmuEcO3XAdRC0"],
        "cnf": {
            "jwk": {
                "kty": "EC",
                "crv": "P-256",
                "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
            }
        }
    }), Ok(()))]
    #[case(json!({
        "vct": "com:example:pid:1",
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "_sd": [0],
        "cnf": {
            "jwk": {
                "kty": "EC",
                "crv": "P-256",
                "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
            }
        }
    }), Err("data did not match any variant of untagged enum ClaimValue".to_owned()))]
    #[case(json!({
        "vct": "com:example:pid:1",
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "nested": {
            "_sd": [0]
        },
        "cnf": {
            "jwk": {
                "kty": "EC",
                "crv": "P-256",
                "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
            }
        }
    }), Err("data did not match any variant of untagged enum ClaimValue".to_owned()))]
    #[case(json!({
        "vct": "com:example:pid:1",
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "array": [{
            "_sd": [0]
        }],
        "cnf": {
            "jwk": {
                "kty": "EC",
                "crv": "P-256",
                "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
            }
        }
    }), Err("data did not match any variant of untagged enum ClaimValue".to_owned()))]
    #[case(json!({
        "vct": "com:example:pid:1",
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "array": [{ "...": 0 }],
        "cnf": {
            "jwk": {
                "kty": "EC",
                "crv": "P-256",
                "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
            }
        }
    }), Err("data did not match any variant of untagged enum ClaimValue".to_owned()))]
    #[case(json!({
        "vct": "com:example:pid:1",
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "...": "not_allowed",
        "cnf": {
            "jwk": {
                "kty": "EC",
                "crv": "P-256",
                "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
            }
        }
    }), Err("data did not match any variant of untagged enum ClaimValue".to_owned()))]
    #[case(json!({
        "vct": "com:example:pid:1",
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "nationalities":
        ["DE", {"...":"w0I8EKcdCtUPkGCNUrfwVp2xEgNjtoIDlOxc9-PlOhs"}, "US"],
        "cnf": {
            "jwk": {
                "kty": "EC",
                "crv": "P-256",
                "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
            }
        }
    }), Ok(()))]
    #[case(json!({
        "vct": "com:example:pid:1",
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "family_name": "Möbius",
        "nationalities": [
            { "...": "PmnlrRjhLcwf8zTDdK15HVGwHtPYjddvD362WjBLwro" },
            { "...": "r823HFN6Ba_lpSANYtXqqCBAH-TsQlIzfOK0lRAFLCM" },
            { "...": "nP5GYjwhFm6ESlAeC4NCaIliW4tz0hTrUeoJB3lb5TA" }
        ],
        "cnf": {
            "jwk": {
                "kty": "EC",
                "crv": "P-256",
                "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
            }
        }
    }), Ok(()))]
    #[case(json!({
        "vct": "com:example:pid:1",
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
    }), Ok(()))]
    #[case(json!({
        "vct": "com:example:pid:1",
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
        "cnf": {
            "jwk": {
                "kty": "EC",
                "crv": "P-256",
                "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
            }
        },
        "_sd_alg": "sha-256"
    }), Ok(()))]
    #[case(json!({
        "vct": "com:example:pid:1",
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
        "_sd_alg": "sha-256",
        "cnf": {
            "jwk": {
                "kty": "EC",
                "crv": "P-256",
                "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
            }
        }
    }
    ), Ok(()))]
    #[case(json!({
        "vct": "com:example:pid:1",
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
        "_sd_alg": "sha-256",
        "cnf": {
            "jwk": {
                "kty": "EC",
                "crv": "P-256",
                "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
            }
        }
    }), Ok(()))]
    fn test_different_serialization_scenarios(
        #[case] original: serde_json::Value,
        #[case] expected: std::result::Result<(), String>,
    ) {
        let result = serde_json::from_value::<SdJwtVcClaims>(original.clone());
        match (result, expected) {
            (Ok(r), Ok(())) => {
                let serialized = serde_json::to_value(r).unwrap();
                assert_eq!(serialized, original);
            }
            (Err(e), Err(r)) => assert_eq!(e.to_string(), r.to_string()),
            (Err(e), Ok(())) => {
                panic!("assertion failed\n left: {e}\nright: Ok")
            }
            (Ok(r), Err(e)) => {
                panic!("assertion failed\n left: {r:?}\nright: {e}")
            }
        };
    }

    #[test]
    fn sd_jwt_claims_features() {
        let value = json!({
            "_sd": [
                "CrQe7S5kqBAHt-nMYXgc6bdt2SH5aTY1sU_M-PgkjPI",
            ],
            "iss": "https://issuer.example.com/",
            "vct": "com:example:pid:1",
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
        let parsed: SdJwtVcClaims = serde_json::from_value(value).unwrap();
        let expected = SdJwtVcClaims {
            cnf: RequiredKeyBinding::Jwk(Jwk {
                common: Default::default(),
                algorithm: AlgorithmParameters::EllipticCurve(EllipticCurveKeyParameters {
                    curve: EllipticCurve::P256,
                    key_type: EllipticCurveKeyType::EC,
                    x: "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc".to_string(),
                    y: "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ".to_string(),
                }),
            }),
            _sd_alg: Some(SdAlg::Sha256),
            vct_integrity: None,
            iss: "https://issuer.example.com/".parse::<HttpsUri>().unwrap(),
            iat: DateTimeSeconds::new(DateTime::from_timestamp(1683000000, 0).unwrap()),
            exp: DateTime::from_timestamp(1883000000, 0).map(DateTimeSeconds::new),
            nbf: None,
            vct: "com:example:pid:1".to_string(),
            claims: ClaimValue::Object(ObjectClaims {
                _sd: Some(
                    vec!["CrQe7S5kqBAHt-nMYXgc6bdt2SH5aTY1sU_M-PgkjPI".to_string()]
                        .try_into()
                        .unwrap(),
                ),
                claims: IndexMap::from([
                    ("sub".parse().unwrap(), ClaimValue::String("user_42".to_string())),
                    (
                        "object_with_digests".parse().unwrap(),
                        ClaimValue::Object(ObjectClaims {
                            _sd: Some(
                                vec!["gbOsI4Edq2x2Kw-w5wPEzakob9hV1cRD0ATN3oQL9JM".to_string()]
                                    .try_into()
                                    .unwrap(),
                            ),
                            claims: IndexMap::from([(
                                "field".parse().unwrap(),
                                ClaimValue::String("value".to_string()),
                            )]),
                        }),
                    ),
                    (
                        "object_with_array_of_digests".parse().unwrap(),
                        ClaimValue::Object(ObjectClaims {
                            _sd: None,
                            claims: IndexMap::from([(
                                "array".parse().unwrap(),
                                ClaimValue::Array(vec![ArrayClaim::Hash {
                                    digest: "pFndjkZ_VCzmyTa6UjlZo3dh-ko8aIKQc9DlGzhaVYo".to_string(),
                                }]),
                            )]),
                        }),
                    ),
                    (
                        "array_of_digests".parse().unwrap(),
                        ClaimValue::Array(vec![ArrayClaim::Hash {
                            digest: "pFndjkZ_VCzmyTa6UjlZo3dh-ko8aIKQc9DlGzhaVYo".to_string(),
                        }]),
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
                                claims: IndexMap::new(),
                            })),
                            ArrayClaim::Hash {
                                digest: "7Cf6JkPudry3lcbwHgeZ8khAv1U1OSlerP0VkBJrWZ0".to_string(),
                            },
                        ]),
                    ),
                ]),
            }),
            attestation_qualification: None,
            status: None,
        };
        assert_eq!(parsed, expected);
    }

    fn parse_and_verify_disclosures(
        object: Value,
        disclosures: Vec<Disclosure>,
    ) -> Result<IndexMap<String, Disclosure>, DecoderError> {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_keypair = ca.generate_issuer_mock().unwrap();
        let holder_key = SigningKey::random(&mut OsRng);

        let sd_jwt = SdJwtBuilder::new(SdJwtVcClaims::example_from_json(
            holder_key.verifying_key(),
            json!({}),
            &MockTimeGenerator::default(),
        ))
        .finish(&issuer_keypair)
        .now_or_never()
        .unwrap()
        .unwrap()
        .to_string();

        let mut parsed = sd_jwt.parse::<UnverifiedSdJwt>().unwrap();
        parsed.disclosures = disclosures.into_iter().map(|d| d.to_string()).collect();

        let claims: SdJwtExampleClaims = serde_json::from_value(object).unwrap();
        UnverifiedSdJwt::<SdJwtExampleClaims, ()>::parse_and_verify_disclosures(&parsed.disclosures, &claims)
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

        assert_matches!(error, DecoderError::UnreferencedDisclosure(digest) if digest == root_object_claim_digest);
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

        assert_matches!(error, DecoderError::ClaimStructure(ClaimError::DisclosureTypeMismatch {
            expected, actual, digest, }) if
            expected == ClaimType::Array && actual == ClaimType::Object && digest == array_claim_digest );
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

        assert_matches!(error, DecoderError::ClaimStructure(ClaimError::DisclosureTypeMismatch {
            expected, actual, digest, }) if
            expected == ClaimType::Object && actual == ClaimType::Array && digest == object_claim_digest );
    }
}
