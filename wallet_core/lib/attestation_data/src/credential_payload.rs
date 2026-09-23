use std::collections::HashMap;

use attestation_types::status_claim::StatusClaim;
use attestation_types::status_claim::StatusListClaim;
use chrono::DateTime;
use chrono::Utc;
use cose::CoseError;
use cose::CoseKey;
use cose::TypedCose;
use coset::CoseSign1;
use crypto::EcdsaKey;
use crypto::PublicKey;
use crypto::server_keys::KeyPair;
use error_category::ErrorCategory;
use jwt::confirmation::ConfirmationClaim;
use jwt::error::JwkConversionError;
use jwt::jwk::jwk_from_public_key;
use mdoc::DeviceKeyInfo;
use mdoc::DigestAlgorithm;
use mdoc::IssuerNameSpaces;
use mdoc::IssuerNameSpacesPreConditionError;
use mdoc::IssuerSigned;
use mdoc::MdocStatus;
use mdoc::MobileSecurityObject;
use mdoc::MobileSecurityObjectVersion;
use mdoc::holder::Mdoc;
use mdoc::utils::crypto::CryptoError;
use mdoc::utils::serialization::CborError;
use mdoc::utils::serialization::TaggedBytes;
use p256::ecdsa::VerifyingKey;
use sd_jwt::builder::SdJwtBuilder;
use sd_jwt::builder::SignedSdJwt;
use sd_jwt::sd_jwt::SdJwtVcClaims;
use sd_jwt::sd_jwt::VerifiedSdJwt;
use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use ssri::Integrity;
use utils::date_time_seconds::DateTimeSeconds;
use utils::generator::Generator;

use crate::attributes::Attributes;
use crate::attributes::AttributesError;
use crate::attributes::AttributesTraversalBehaviour;
use crate::attributes::ClaimValueError;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum PreviewableCredentialPayloadFromSdJwtError {
    #[error("error converting from SD-JWT: {0}")]
    #[category(pd)]
    SdJwtDecoding(#[source] sd_jwt::error::DecoderError),

    #[error("error converting claims to attributes: {0}")]
    #[category(pd)]
    InvalidAttributes(#[source] AttributesError),
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum PreviewableCredentialPayloadFromMdocError {
    #[error("unable to convert mdoc TDate to DateTime<Utc>")]
    #[category(critical)]
    DateConversion(#[source] chrono::ParseError),

    #[error("attributes error: {0}")]
    #[category(pd)]
    InvalidAttributes(#[source] AttributesError),
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewableCredentialPayload {
    #[serde(rename = "vct")]
    pub attestation_type: String,

    #[serde(rename = "exp")]
    pub expires: Option<DateTimeSeconds>,

    #[serde(rename = "nbf")]
    pub not_before: Option<DateTimeSeconds>,

    #[serde(flatten)]
    pub attributes: Attributes,
}

impl PreviewableCredentialPayload {
    pub fn matches_existing(
        &self,
        existing: &PreviewableCredentialPayload,
        time: &impl Generator<DateTime<Utc>>,
    ) -> bool {
        // Compare all fields except `not_before`
        if self.attestation_type == existing.attestation_type && self.attributes == existing.attributes {
            // - If `not_before` are equal as well, they definitely match
            // - If not, it is only considered a match if `not_before` from the new preview (self) is in the past
            return self.not_before == existing.not_before
                || self.not_before.is_some_and(|nbf| nbf.as_ref() <= &time.generate());
        }

        false
    }

    pub fn from_sd_jwt(sd_jwt: VerifiedSdJwt) -> Result<Self, PreviewableCredentialPayloadFromSdJwtError> {
        Ok(SplitSdJwtCredential::from_sd_jwt(sd_jwt)?.previewable)
    }

    pub fn from_mdoc(mdoc: Mdoc) -> Result<Self, PreviewableCredentialPayloadFromMdocError> {
        Ok(SplitMdocCredential::from_mdoc(mdoc)?.previewable)
    }
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum CredentialPayloadFromSdJwtError {
    #[error("error converting SD-JWT to PreviewableCredentialPayload: {0}")]
    #[category(defer)]
    Previewable(#[source] PreviewableCredentialPayloadFromSdJwtError),

    #[error("missing status claim")]
    #[category(critical)]
    MissingStatusClaim,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum CredentialPayloadIntoSignedSdJwtError {
    #[error("error converting AttributeName to ClaimName: {0}")]
    #[category(pd)]
    InvalidClaimValue(#[source] ClaimValueError),

    #[error("error converting to SD-JWT: {0}")]
    #[category(pd)]
    SdJwtEncoding(#[source] sd_jwt::error::EncoderError),

    #[error("missing status claim")]
    #[category(critical)]
    MissingStatusClaim,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum CredentialPayloadFromMdocError {
    #[error("error converting mdoc to PreviewableCredentialPayload: {0}")]
    #[category(defer)]
    Previewable(#[source] PreviewableCredentialPayloadFromMdocError),

    #[error("error converting holder public CoseKey to a VerifyingKey: {0}")]
    #[category(pd)]
    CoseKeyConversion(#[source] CryptoError),

    #[error("error converting holder VerifyingKey to JWK: {0}")]
    #[category(pd)]
    JwkConversion(#[source] JwkConversionError),

    #[error("missing status claim")]
    #[category(critical)]
    MissingStatusClaim,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum CredentialPayloadIntoSignedMdocError {
    #[error("missing valid_from in mdoc validity info")]
    #[category(critical)]
    MissingValidFrom,

    #[error("missing valid_until in mdoc validity info")]
    #[category(critical)]
    MissingValidUntil,

    #[error("cannot convert attributes to mdoc: {0}")]
    #[category(pd)]
    InvalidAttributes(#[source] AttributesError),

    #[error("missing status claim")]
    #[category(critical)]
    MissingStatusClaim,

    #[error("missing or empty NameSpace detected: {0}")]
    #[category(critical)]
    MissingOrEmptyNamespace(#[source] IssuerNameSpacesPreConditionError),

    #[error("error converting holder VerifyingKey to JWK: {0}")]
    #[category(pd)]
    JwkConversion(#[source] JwkConversionError),

    #[error("error converting holder public CoseKey to a VerifyingKey: {0}")]
    #[category(pd)]
    CoseKeyConversion(#[source] CryptoError),

    #[error("error converting issuer namespaces to CBOR: {0}")]
    #[category(pd)]
    CborConversion(#[source] CborError),

    #[error("error signing mdoc: {0}")]
    #[category(pd)]
    SigningError(#[source] CoseError),

    #[error("unsupported confirmation key: {0:?}")]
    #[category(pd)]
    UnsupportedConfirmationKey(Box<PublicKey>),
}

/// This struct represents the Claims Set received from the issuer, as a format-agnostic representation shared by
/// both mdoc and SD-JWT.
///
/// Converting both an (unsigned) mdoc and SD-JWT document to this struct should yield the same result.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CredentialPayload {
    #[serde(rename = "iat")]
    pub issued_at: DateTimeSeconds,

    /// Contains the attestation's public key, of which the corresponding private key is used by the wallet during
    /// disclosure to sign the RP's nonce into a PoP
    #[serde(rename = "cnf")]
    pub confirmation_key: ConfirmationClaim,

    /// Contains the integrity digest of the SD-JWT Type Metadata document of this `vct`. When absent, the credential
    /// is described by the issuer credential metadata instead.
    #[serde(rename = "vct#integrity")]
    pub vct_integrity: Option<Integrity>,

    /// The information on how to read the status of the Verifiable Credential. Every credential is required to
    /// declare a status claim. `None` is only used to be able to handle mdocs that specify an
    /// `identifier_list`, which is not yet supported (PVW-6106)
    pub status: Option<StatusListClaim>,

    #[serde(flatten)]
    pub previewable_payload: PreviewableCredentialPayload,
}

impl CredentialPayload {
    pub fn from_previewable_credential_payload(
        previewable_payload: PreviewableCredentialPayload,
        issued_at: DateTime<Utc>,
        holder_pubkey: &PublicKey,
        vct_integrity: Option<Integrity>,
        status: Option<StatusListClaim>,
    ) -> Result<Self, JwkConversionError> {
        let confirmation_key = jwk_from_public_key(holder_pubkey)?;

        Ok(CredentialPayload {
            issued_at: issued_at.into(),
            confirmation_key: ConfirmationClaim::Jwk(confirmation_key),
            vct_integrity,
            status,
            previewable_payload,
        })
    }

    pub fn from_sd_jwt(sd_jwt: VerifiedSdJwt) -> Result<Self, CredentialPayloadFromSdJwtError> {
        let split = SplitSdJwtCredential::from_sd_jwt(sd_jwt).map_err(CredentialPayloadFromSdJwtError::Previewable)?;

        let StatusClaim::StatusList(status) = split
            .status
            .ok_or(CredentialPayloadFromSdJwtError::MissingStatusClaim)?;

        Ok(CredentialPayload {
            issued_at: split.issued_at,
            confirmation_key: split.confirmation_key,
            vct_integrity: split.vct_integrity,
            status: Some(status),
            previewable_payload: split.previewable,
        })
    }

    pub fn from_mdoc(mdoc: Mdoc) -> Result<Self, CredentialPayloadFromMdocError> {
        let split = SplitMdocCredential::from_mdoc(mdoc).map_err(CredentialPayloadFromMdocError::Previewable)?;

        let confirmation_key = ConfirmationClaim::Jwk(
            jwk_from_public_key(&PublicKey::from(
                VerifyingKey::try_from(split.device_key_info)
                    .map_err(CredentialPayloadFromMdocError::CoseKeyConversion)?,
            ))
            .map_err(CredentialPayloadFromMdocError::JwkConversion)?,
        );
        // A status claim must be present, but `identifier_list` status are ignored for now (PVW-6106)
        let status = split.status.ok_or(CredentialPayloadFromMdocError::MissingStatusClaim)?;
        let status = status_list_claim_from_mdoc_status(status);

        Ok(CredentialPayload {
            issued_at: split.issued_at,
            confirmation_key,
            vct_integrity: None,
            status,
            previewable_payload: split.previewable,
        })
    }

    pub async fn into_signed_sd_jwt(
        self,
        type_metadata: &NormalizedTypeMetadata,
        issuer_keypair: &KeyPair<impl EcdsaKey>,
    ) -> Result<SignedSdJwt, CredentialPayloadIntoSignedSdJwtError> {
        if self.status.is_none() {
            return Err(CredentialPayloadIntoSignedSdJwtError::MissingStatusClaim);
        }

        let sd_by_claims = type_metadata
            .claims()
            .iter()
            .map(|claim| (&claim.path, claim.sd))
            .collect::<HashMap<_, _>>();

        let sd_jwt = self
            .previewable_payload
            .attributes
            .claim_paths(AttributesTraversalBehaviour::AllPaths)
            .into_iter()
            .try_fold(
                SdJwtBuilder::new(
                    self.try_into()
                        .map_err(CredentialPayloadIntoSignedSdJwtError::InvalidClaimValue)?,
                ),
                |builder, claims| {
                    let should_be_selectively_disclosable = match sd_by_claims.get(&claims) {
                        Some(sd) => !matches!(sd, ClaimSelectiveDisclosureMetadata::Never),
                        None => true,
                    };

                    if !should_be_selectively_disclosable {
                        return Ok(builder);
                    }

                    builder
                        .make_concealable(claims)
                        .map_err(CredentialPayloadIntoSignedSdJwtError::SdJwtEncoding)
                },
            )?
            .finish(issuer_keypair)
            .await
            .map_err(CredentialPayloadIntoSignedSdJwtError::SdJwtEncoding)?;

        Ok(sd_jwt)
    }

    pub async fn into_signed_mdoc(
        self,
        issuer_keypair: &KeyPair<impl EcdsaKey>,
    ) -> Result<(IssuerSigned, MobileSecurityObject), CredentialPayloadIntoSignedMdocError> {
        let CredentialPayload {
            issued_at,
            confirmation_key,
            status,
            previewable_payload,
            ..
        } = self;
        let PreviewableCredentialPayload {
            not_before,
            expires,
            attributes,
            attestation_type,
        } = previewable_payload;

        let status = status.ok_or(CredentialPayloadIntoSignedMdocError::MissingStatusClaim)?;

        let validity = mdoc::ValidityInfo {
            signed: issued_at.into(),
            valid_from: not_before
                .map(Into::into)
                .ok_or(CredentialPayloadIntoSignedMdocError::MissingValidFrom)?,
            valid_until: expires
                .map(Into::into)
                .ok_or(CredentialPayloadIntoSignedMdocError::MissingValidUntil)?,
            expected_update: None,
        };

        let attributes = attributes
            .to_mdoc_attributes()
            .map_err(CredentialPayloadIntoSignedMdocError::InvalidAttributes)?;
        let attrs = IssuerNameSpaces::try_from(attributes)
            .map_err(CredentialPayloadIntoSignedMdocError::MissingOrEmptyNamespace)?;

        let doc_type = attestation_type;
        let pubkey = confirmation_key
            .try_to_public_key()
            .map_err(CredentialPayloadIntoSignedMdocError::JwkConversion)?;
        let PublicKey::ESP256(verifying_key) = pubkey else {
            return Err(CredentialPayloadIntoSignedMdocError::UnsupportedConfirmationKey(
                Box::new(pubkey),
            ));
        };

        let cose_pubkey: CoseKey = (&verifying_key)
            .try_into()
            .map_err(CryptoError::from)
            .map_err(CredentialPayloadIntoSignedMdocError::CoseKeyConversion)?;

        let mso = MobileSecurityObject {
            version: MobileSecurityObjectVersion::V1_0,
            digest_algorithm: DigestAlgorithm::SHA256,
            doc_type,
            value_digests: (&attrs)
                .try_into()
                .map_err(CredentialPayloadIntoSignedMdocError::CborConversion)?,
            device_key_info: cose_pubkey.into(),
            validity_info: validity,
            status: Some(MdocStatus::StatusList(status)),
        };

        let mso = TaggedBytes(mso);
        let issuer_auth: TypedCose<CoseSign1, TaggedBytes<MobileSecurityObject>> =
            TypedCose::sign_with_certificate(&mso, issuer_keypair, true)
                .await
                .map_err(CredentialPayloadIntoSignedMdocError::SigningError)?;
        let TaggedBytes(mso) = mso;

        Ok((
            IssuerSigned {
                name_spaces: attrs.into(),
                issuer_auth,
            },
            mso,
        ))
    }
}

// Eerything needed for a [`PreviewableCredentialPayload`], plus the remaining claims needed to build the full
// [`CredentialPayload`] without decoding the SD-JWT again.
struct SplitSdJwtCredential {
    previewable: PreviewableCredentialPayload,
    issued_at: DateTimeSeconds,
    confirmation_key: ConfirmationClaim,
    vct_integrity: Option<Integrity>,
    status: Option<StatusClaim>,
}

impl SplitSdJwtCredential {
    fn from_sd_jwt(sd_jwt: VerifiedSdJwt) -> Result<Self, PreviewableCredentialPayloadFromSdJwtError> {
        let attributes = sd_jwt
            .decoded_claims()
            .map_err(PreviewableCredentialPayloadFromSdJwtError::SdJwtDecoding)?
            .try_into()
            .map_err(PreviewableCredentialPayloadFromSdJwtError::InvalidAttributes)?;
        let claims = sd_jwt.into_claims();

        let previewable = PreviewableCredentialPayload {
            attestation_type: claims.vct,
            expires: claims.exp,
            not_before: claims.nbf,
            attributes,
        };

        Ok(SplitSdJwtCredential {
            previewable,
            issued_at: claims.iat,
            confirmation_key: claims.cnf,
            vct_integrity: claims.vct_integrity,
            status: claims.status,
        })
    }
}

// Everything needed for a [`PreviewableCredentialPayload`], plus the remaining MSO fields needed to build the full
// [`CredentialPayload`]. The holder public key conversion (the expensive part) is deferred until it's actually needed.
struct SplitMdocCredential {
    previewable: PreviewableCredentialPayload,
    issued_at: DateTimeSeconds,
    device_key_info: DeviceKeyInfo,
    status: Option<MdocStatus>,
}

impl SplitMdocCredential {
    fn from_mdoc(mdoc: Mdoc) -> Result<Self, PreviewableCredentialPayloadFromMdocError> {
        let (mso, issuer_signed) = mdoc.into_components();
        let attributes = issuer_signed.into_entries_by_namespace();
        let attestation_type = mso.doc_type;
        let attributes = Attributes::from_mdoc_attributes(attributes)
            .map_err(PreviewableCredentialPayloadFromMdocError::InvalidAttributes)?;

        let issued_at = (&mso.validity_info.signed)
            .try_into()
            .map_err(PreviewableCredentialPayloadFromMdocError::DateConversion)?;

        let previewable = PreviewableCredentialPayload {
            attestation_type,
            expires: Some(
                (&mso.validity_info.valid_until)
                    .try_into()
                    .map_err(PreviewableCredentialPayloadFromMdocError::DateConversion)?,
            ),
            not_before: Some(
                (&mso.validity_info.valid_from)
                    .try_into()
                    .map_err(PreviewableCredentialPayloadFromMdocError::DateConversion)?,
            ),
            attributes,
        };

        Ok(SplitMdocCredential {
            previewable,
            issued_at,
            device_key_info: mso.device_key_info,
            status: mso.status,
        })
    }
}

// TODO this should be removed once `identifier_list` status are supported (PVW-6106)
fn status_list_claim_from_mdoc_status(status: MdocStatus) -> Option<StatusListClaim> {
    match status {
        MdocStatus::StatusList(claim) => Some(claim),
        MdocStatus::IdentifierList(_) => None,
    }
}

impl TryFrom<CredentialPayload> for SdJwtVcClaims {
    type Error = ClaimValueError;

    fn try_from(value: CredentialPayload) -> Result<Self, Self::Error> {
        Ok(SdJwtVcClaims {
            vct: value.previewable_payload.attestation_type,
            vct_integrity: value.vct_integrity,
            iat: value.issued_at,
            exp: value.previewable_payload.expires,
            nbf: value.previewable_payload.not_before,
            cnf: value.confirmation_key,
            status: value.status.map(StatusClaim::StatusList),
            _sd_alg: None, // TODO this should be handled elsewhere (PVW-5121)

            claims: value.previewable_payload.attributes.try_into()?,
        })
    }
}

#[cfg(any(test, feature = "example_credential_payloads"))]
mod examples {
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use chrono::DateTime;
    use chrono::Duration;
    use chrono::Utc;
    use crypto::PublicKey;
    use jwt::jwk::jwk_from_public_key;
    use p256::ecdsa::VerifyingKey;
    use ssri::Integrity;
    use utils::generator::Generator;

    use super::*;
    use crate::attributes::Attribute;
    use crate::attributes::Attributes;

    impl CredentialPayload {
        pub fn from_previewable_credential_payload_unvalidated(
            previewable_payload: PreviewableCredentialPayload,
            issued_at: DateTime<Utc>,
            holder_pubkey: &PublicKey,
            vct_integrity: Option<Integrity>,
            status: Option<StatusListClaim>,
        ) -> Result<Self, JwkConversionError> {
            Ok(Self {
                issued_at: issued_at.into(),
                confirmation_key: ConfirmationClaim::try_from_public_key(holder_pubkey)?,
                vct_integrity,
                status,
                previewable_payload,
            })
        }

        pub(super) fn example_with_preview(
            previewable_payload: PreviewableCredentialPayload,
            verifying_key: &VerifyingKey,
            time_generator: &impl Generator<DateTime<Utc>>,
        ) -> Self {
            let time = time_generator.generate();

            let confirmation_key = jwk_from_public_key(&PublicKey::from(*verifying_key)).unwrap();

            Self {
                issued_at: time.into(),
                confirmation_key: ConfirmationClaim::Jwk(confirmation_key),
                vct_integrity: Some(Integrity::from("")),
                status: Some(StatusListClaim::new_mock()),
                previewable_payload,
            }
        }

        pub fn example_with_attributes(
            attestation_type: &str,
            attributes: Attributes,
            verifying_key: &VerifyingKey,
            time_generator: &impl Generator<DateTime<Utc>>,
        ) -> Self {
            let previewable_payload =
                PreviewableCredentialPayload::example_with_attributes(attestation_type, attributes, time_generator);

            Self::example_with_preview(previewable_payload, verifying_key, time_generator)
        }
    }

    impl PreviewableCredentialPayload {
        pub fn example_empty(attestation_type: &str, time_generator: &impl Generator<DateTime<Utc>>) -> Self {
            let time = time_generator.generate();

            Self {
                attestation_type: attestation_type.to_string(),
                expires: Some((time + Duration::days(365)).into()),
                not_before: Some((time - Duration::days(1)).into()),
                attributes: Attributes::default(),
            }
        }

        pub fn example_family_name(time_generator: &impl Generator<DateTime<Utc>>) -> Self {
            Self::example_with_attributes(
                PID_ATTESTATION_TYPE,
                Attributes::example([(["family_name"], Attribute::Text(String::from("De Bruijn")))]),
                time_generator,
            )
        }

        pub fn example_family_name_mdoc(time_generator: &impl Generator<DateTime<Utc>>) -> Self {
            Self::example_with_attributes(
                PID_ATTESTATION_TYPE,
                Attributes::example([(
                    [PID_ATTESTATION_TYPE, "family_name"],
                    Attribute::Text(String::from("De Bruijn")),
                )]),
                time_generator,
            )
        }

        pub fn example_with_attributes(
            attestation_type: &str,
            attributes: Attributes,
            time_generator: &impl Generator<DateTime<Utc>>,
        ) -> Self {
            Self {
                attributes,
                ..Self::example_empty(attestation_type, time_generator)
            }
        }
    }
}

#[cfg(feature = "mock")]
mod mock {
    use attestation_types::pid_constants::ADDRESS_ATTESTATION_TYPE;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use chrono::DateTime;
    use chrono::Utc;
    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::Generate;
    use utils::generator::Generator;

    use super::CredentialPayload;
    use super::PreviewableCredentialPayload;
    use crate::attributes::Attributes;

    impl CredentialPayload {
        pub fn nl_pid_example(time_generator: &impl Generator<DateTime<Utc>>) -> (Self, SigningKey) {
            let previewable_payload = PreviewableCredentialPayload::nl_pid_example(time_generator);
            let holder_key = SigningKey::generate();
            (
                Self::example_with_preview(previewable_payload, holder_key.verifying_key(), time_generator),
                holder_key,
            )
        }

        /// The same claims as [`CredentialPayload::nl_pid_example()`], but with mdoc-namespaced attributes.
        pub fn nl_pid_mdoc_example(time_generator: &impl Generator<DateTime<Utc>>) -> (Self, SigningKey) {
            let previewable_payload = PreviewableCredentialPayload::nl_pid_mdoc_example(time_generator);
            let holder_key = SigningKey::generate();
            (
                Self::example_with_preview(previewable_payload, holder_key.verifying_key(), time_generator),
                holder_key,
            )
        }

        pub fn nl_pid_address_example(time_generator: &impl Generator<DateTime<Utc>>) -> Self {
            let previewable_payload = PreviewableCredentialPayload::nl_pid_address_example(time_generator);

            Self::example_with_preview(
                previewable_payload,
                SigningKey::generate().verifying_key(),
                time_generator,
            )
        }

        /// The same claims as [`CredentialPayload::nl_pid_address_example()`], but with mdoc-namespaced attributes.
        pub fn nl_pid_address_mdoc_example(time_generator: &impl Generator<DateTime<Utc>>) -> Self {
            let previewable_payload = PreviewableCredentialPayload::nl_pid_address_mdoc_example(time_generator);

            Self::example_with_preview(
                previewable_payload,
                SigningKey::generate().verifying_key(),
                time_generator,
            )
        }
    }

    impl PreviewableCredentialPayload {
        pub fn nl_pid_example(time_generator: &impl Generator<DateTime<Utc>>) -> Self {
            Self::example_with_attributes(PID_ATTESTATION_TYPE, Attributes::nl_pid_example(), time_generator)
        }

        /// The same claims as [`PreviewableCredentialPayload::nl_pid_example()`], but with mdoc-namespaced
        /// attributes.
        pub fn nl_pid_mdoc_example(time_generator: &impl Generator<DateTime<Utc>>) -> Self {
            Self::example_with_attributes(PID_ATTESTATION_TYPE, Attributes::nl_pid_mdoc_example(), time_generator)
        }

        pub fn nl_pid_address_example(time_generator: &impl Generator<DateTime<Utc>>) -> Self {
            Self::example_with_attributes(
                ADDRESS_ATTESTATION_TYPE,
                Attributes::nl_pid_address_example(),
                time_generator,
            )
        }

        /// The same claims as [`PreviewableCredentialPayload::nl_pid_address_example()`], but with mdoc-namespaced
        /// attributes. Note that the address is not grouped, as an mdoc has no nesting of its own.
        pub fn nl_pid_address_mdoc_example(time_generator: &impl Generator<DateTime<Utc>>) -> Self {
            Self::example_with_attributes(
                ADDRESS_ATTESTATION_TYPE,
                Attributes::nl_pid_address_mdoc_example(),
                time_generator,
            )
        }
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;
    use std::sync::Arc;
    use std::time::Duration;

    use attestation_types::claim_path::ClaimPath;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use attestation_types::status_claim::IdentifierListInfo;
    use chrono::TimeZone;
    use chrono::Utc;
    use crypto::PublicKey;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::mock_remote::MockRemoteWscd;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use futures::FutureExt;
    use indexmap::IndexMap;
    use itertools::Itertools;
    use jwt::jwk::jwk_from_public_key;
    use jwt::nonce::Nonce;
    use mdoc::MdocStatus;
    use mdoc::holder::Mdoc;
    use mdoc::utils::serialization::TaggedBytes;
    use mdoc::verifier::ValidityRequirement;
    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::Generate;
    use sd_jwt::builder::SdJwtBuilder;
    use sd_jwt::key_binding_jwt::KbVerificationOptions;
    use sd_jwt::key_binding_jwt::KeyBindingJwtBuilder;
    use sd_jwt::sd_jwt::SdJwtVcClaims;
    use sd_jwt::sd_jwt::UnsignedSdJwtPresentation;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::UncheckedTypeMetadata;
    use serde_json::json;
    use ssri::Integrity;
    use token_status_list::verification::client::mock::StatusListClientStub;
    use token_status_list::verification::verifier::RevocationVerifier;
    use utils::generator::TimeGenerator;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_nonempty;

    use super::*;
    use crate::attributes::Attribute;
    use crate::attributes::Attributes;
    use crate::attributes::test::complex_attributes;
    use crate::auth::issuer_auth::IssuerRegistration;
    use crate::x509::generate::mock::generate_issuer_mock_with_registration;

    fn setup_into_signed() -> (
        PreviewableCredentialPayload,
        CredentialPayload,
        NormalizedTypeMetadata,
        Integrity,
        Ca,
        KeyPair,
    ) {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuance_key = ca.generate_pid_issuer_mock().unwrap();

        let payload_preview = PreviewableCredentialPayload::example_with_attributes(
            PID_ATTESTATION_TYPE,
            Attributes::example([
                (["first_name"], Attribute::Text("John".to_string())),
                (["family_name"], Attribute::Text("Doe".to_string())),
            ]),
            &MockTimeGenerator::default(),
        );

        // Note that this resource integrity does not match any metadata source document.
        let metadata_integrity = Integrity::from(crypto::utils::random_bytes(32));
        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::example_with_claim_names(
            PID_ATTESTATION_TYPE,
            &["first_name", "family_name"],
        ));
        let credential_payload = CredentialPayload::from_previewable_credential_payload(
            payload_preview.clone(),
            Utc::now(),
            &PublicKey::from(*SigningKey::generate().verifying_key()),
            Some(metadata_integrity.clone()),
            Some(StatusListClaim::new_mock()),
        )
        .unwrap();

        (
            payload_preview,
            credential_payload,
            metadata,
            metadata_integrity,
            ca,
            issuance_key,
        )
    }

    #[tokio::test]
    async fn test_into_signed_mdoc() {
        let (payload_preview, credential_payload, _, _, ca, issuance_key) = setup_into_signed();

        // The attributes of an mdoc are laid out in namespaces, as the issuer authors them.
        let credential_payload = CredentialPayload {
            previewable_payload: PreviewableCredentialPayload {
                attributes: Attributes::example([
                    (
                        [PID_ATTESTATION_TYPE, "first_name"],
                        Attribute::Text("John".to_string()),
                    ),
                    (
                        [PID_ATTESTATION_TYPE, "family_name"],
                        Attribute::Text("Doe".to_string()),
                    ),
                ]),
                ..payload_preview.clone()
            },
            ..credential_payload
        };

        let (issuer_signed, _) = credential_payload.into_signed_mdoc(&issuance_key).await.unwrap();

        // The IssuerSigned should be valid
        issuer_signed
            .verify(ValidityRequirement::Valid, &TimeGenerator, &TrustAnchors::from(&ca))
            .expect("the IssuerSigned sent in the preview should be valid");

        // The issuer certificate generated above should be included in the IssuerAuth
        assert_eq!(
            &issuer_signed.issuer_auth.x5chain().unwrap().into_first(),
            issuance_key.certificate()
        );

        let TaggedBytes(cose_payload) = issuer_signed.issuer_auth.dangerous_parse_unverified().unwrap();
        assert_eq!(cose_payload.doc_type, payload_preview.attestation_type);
        assert_eq!(
            payload_preview.not_before.unwrap(),
            (&cose_payload.validity_info.valid_from).try_into().unwrap(),
        );
        assert_eq!(
            payload_preview.expires.unwrap(),
            (&cose_payload.validity_info.valid_until).try_into().unwrap(),
        );
    }

    /// An mdoc data element value may be any CBOR value, including a map, which ISO 7367-2 mVC relies on for e.g.
    /// `chassis_number_info`. Such an element should survive signing, CBOR (de)serialization and conversion back
    /// into `Attributes` unchanged.
    #[tokio::test]
    async fn test_into_signed_mdoc_map_valued_data_element_round_trip() {
        let (payload_preview, credential_payload, _, _, _, issuance_key) = setup_into_signed();

        let attributes = Attributes::example([
            (
                vec![PID_ATTESTATION_TYPE, "registration_number"],
                Attribute::Text("AB-CD-12".to_string()),
            ),
            (
                vec![
                    PID_ATTESTATION_TYPE,
                    "chassis_number_info",
                    "vehicle_identification_number",
                ],
                Attribute::Text("WVWZZZ1JZXW000001".to_string()),
            ),
            (
                vec![PID_ATTESTATION_TYPE, "basic_vehicle_info", "make"],
                Attribute::Text("Volkswagen".to_string()),
            ),
            (
                vec![PID_ATTESTATION_TYPE, "date_of_registration"],
                Attribute::Date(chrono::NaiveDate::from_ymd_opt(2023, 1, 10).unwrap()),
            ),
        ]);

        let credential_payload = CredentialPayload {
            previewable_payload: PreviewableCredentialPayload {
                attributes: attributes.clone(),
                ..payload_preview.clone()
            },
            ..credential_payload
        };

        let (issuer_signed, _) = credential_payload.into_signed_mdoc(&issuance_key).await.unwrap();

        assert_eq!(
            Attributes::from_mdoc_attributes(issuer_signed.into_entries_by_namespace()).unwrap(),
            attributes
        );
    }

    #[tokio::test]
    async fn test_into_signed_sd_jwt() {
        let (payload_preview, credential_payload, metadata, metadata_integrity, ca, issuance_key) = setup_into_signed();

        let signed_sd_jwt = credential_payload
            .into_signed_sd_jwt(&metadata, &issuance_key)
            .await
            .unwrap();

        let unverified_sd_jwt = signed_sd_jwt.into_unverified();

        // The IssuerSigned should be valid
        let verified_sd_jwt = unverified_sd_jwt
            .into_verified_against_trust_anchors(&TrustAnchors::from(&ca), &TimeGenerator)
            .expect("the IssuerSigned sent in the preview should be valid");

        // The issuer certificate generated above should be included in the IssuerAuth
        assert_eq!(verified_sd_jwt.issuer_leaf_certificate(), issuance_key.certificate());

        let claims = verified_sd_jwt.claims();
        assert_eq!(claims.vct, payload_preview.attestation_type);
        assert_eq!(claims.nbf, payload_preview.not_before);
        assert_eq!(claims.exp, payload_preview.expires);
        assert_eq!(claims.vct_integrity, Some(metadata_integrity));
    }

    /// The attributes of an mdoc are always exactly two levels deep: the namespace, then the element identifier.
    #[test]
    fn test_from_mdoc() {
        let mdoc = Mdoc::new_mock().now_or_never().unwrap();

        let payload = CredentialPayload::from_mdoc(mdoc)
            .expect("creating and validating CredentialPayload from Mdoc should succeed");

        assert_eq!(
            payload
                .previewable_payload
                .attributes
                .flattened()
                .into_iter()
                .map(|(path, value)| (path.into_inner(), value.clone()))
                .collect_vec(),
            vec![
                (
                    vec![PID_ATTESTATION_TYPE, "bsn"],
                    Attribute::Text("999999999".to_string())
                ),
                (
                    vec![PID_ATTESTATION_TYPE, "given_name"],
                    Attribute::Text("Willeke Liselotte".to_string())
                ),
                (
                    vec![PID_ATTESTATION_TYPE, "family_name"],
                    Attribute::Text("De Bruijn".to_string())
                ),
            ]
        );
    }

    #[tokio::test]
    async fn test_from_mdoc_with_identifier_list_status_is_accepted() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let device_key = MockRemoteEcdsaKey::new("identifier".to_owned(), SigningKey::generate());
        let status = MdocStatus::IdentifierList(IdentifierListInfo {
            id: vec![0xcc, 0xcc],
            uri: "https://example.com/identifierlists/1".parse().unwrap(),
            certificate: None,
        });
        let mdoc = Mdoc::new_mock_with_ca_key_and_status(&ca, &device_key, Some(status)).await;
        let payload = CredentialPayload::from_mdoc(mdoc)
            .expect("creating CredentialPayload from an mdoc with an identifier list status should succeed");

        assert_eq!(payload.status, None);
    }

    #[tokio::test]
    async fn test_from_mdoc_error_missing_status() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let device_key = MockRemoteEcdsaKey::new("identifier".to_owned(), SigningKey::generate());
        let mdoc = Mdoc::new_mock_with_ca_key_and_status(&ca, &device_key, None).await;
        let error = CredentialPayload::from_mdoc(mdoc)
            .expect_err("creating CredentialPayload from an mdoc without a status should fail");

        assert_matches!(error, CredentialPayloadFromMdocError::MissingStatusClaim);
    }

    #[tokio::test]
    async fn test_into_signed_mdoc_error_missing_status() {
        let (_, credential_payload, _, _, _, issuance_key) = setup_into_signed();
        let credential_payload = CredentialPayload {
            status: None,
            ..credential_payload
        };

        let error = credential_payload
            .into_signed_mdoc(&issuance_key)
            .await
            .expect_err("signing a CredentialPayload without a status into an mdoc should fail");

        assert_matches!(error, CredentialPayloadIntoSignedMdocError::MissingStatusClaim);
    }

    #[tokio::test]
    async fn test_into_signed_sd_jwt_error_missing_status() {
        let (_, credential_payload, metadata, _, _, issuance_key) = setup_into_signed();
        let credential_payload = CredentialPayload {
            status: None,
            ..credential_payload
        };

        let error = credential_payload
            .into_signed_sd_jwt(&metadata, &issuance_key)
            .await
            .expect_err("signing a CredentialPayload without a status into an SD-JWT should fail");

        assert_matches!(error, CredentialPayloadIntoSignedSdJwtError::MissingStatusClaim);
    }

    #[test]
    fn test_serialize_deserialize_and_validate() {
        let confirmation_key = jwk_from_public_key(&PublicKey::from(*SigningKey::generate().verifying_key())).unwrap();

        let payload = CredentialPayload {
            issued_at: Utc.with_ymd_and_hms(1970, 1, 1, 0, 1, 1).unwrap().into(),
            confirmation_key: ConfirmationClaim::Jwk(confirmation_key.clone()),
            vct_integrity: Some(Integrity::from("")),
            status: Some(StatusListClaim::new_mock()),
            previewable_payload: PreviewableCredentialPayload {
                attestation_type: String::from("com.example.pid"),
                expires: None,
                not_before: None,
                attributes: complex_attributes().into(),
            },
        };

        let expected_json = json!({
            "vct": "com.example.pid",
            "vct#integrity": "sha256-47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=",
            "iat": 61,
            "status": {
                "idx": 1,
                "uri": "https://example.com/statuslists/1"
            },
            "cnf": {
                "jwk": confirmation_key
            },
            "birth_date": {
                "type": "text",
                "value": "1963-08-12"
            },
            "place_of_birth": {
                "type": "object",
                "value": {
                    "locality": {
                        "type": "text",
                        "value": "The Hague"
                    },
                    "country": {
                        "type": "object",
                        "value": {
                            "name": {
                                "type": "text",
                                "value": "The Netherlands"
                            },
                            "area_code": {
                                "type": "number",
                                "value": 33
                            }
                        }
                    }
                }
            },
            "financial": {
                "type": "object",
                "value": {
                    "has_debt": {
                        "type": "bool",
                        "value": true
                    },
                    "has_job": {
                        "type": "bool",
                        "value": false
                    },
                    "debt_amount": {
                        "type": "number",
                        "value": -10000
                    }
                }
            }
        });

        let json = serde_json::to_value(payload).unwrap();
        assert_eq!(json, expected_json);
    }

    #[test]
    fn test_from_previewable_credential_payload() {
        let holder_key = SigningKey::generate();

        let preview_payload = PreviewableCredentialPayload::example_family_name(&MockTimeGenerator::default());

        let payload = CredentialPayload::from_previewable_credential_payload(
            preview_payload.clone(),
            Utc::now(),
            &PublicKey::from(*holder_key.verifying_key()),
            Some(Integrity::from("")),
            Some(StatusListClaim::new_mock()),
        )
        .unwrap();

        assert_eq!(
            payload.previewable_payload.attestation_type,
            preview_payload.attestation_type,
        );
    }

    #[test]
    fn test_from_sd_jwt() {
        let holder_key = SigningKey::generate();

        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_keypair = ca.generate_issuer_mock().unwrap();

        let claims = SdJwtVcClaims::example_from_json(
            holder_key.verifying_key(),
            json!({
                "birth_date": "1963-08-12",
                "place_of_birth": {
                    "locality": "The Hague",
                    "country": {
                        "name": "The Netherlands",
                        "area_code": 33
                    }
                },
            }),
            &MockTimeGenerator::default(),
        );

        let sd_jwt = SdJwtBuilder::new(claims)
            .make_concealable(vec_nonempty![ClaimPath::SelectByKey(String::from("birth_date"))])
            .unwrap()
            .make_concealable(vec_nonempty![
                ClaimPath::SelectByKey(String::from("place_of_birth")),
                ClaimPath::SelectByKey(String::from("locality")),
            ])
            .unwrap()
            .make_concealable(vec_nonempty![
                ClaimPath::SelectByKey(String::from("place_of_birth")),
                ClaimPath::SelectByKey(String::from("country")),
                ClaimPath::SelectByKey(String::from("name")),
            ])
            .unwrap()
            .make_concealable(vec_nonempty![
                ClaimPath::SelectByKey(String::from("place_of_birth")),
                ClaimPath::SelectByKey(String::from("country")),
                ClaimPath::SelectByKey(String::from("area_code")),
            ])
            .unwrap()
            .add_decoys(&[ClaimPath::SelectByKey(String::from("place_of_birth"))], 1)
            .unwrap()
            .add_decoys(&[], 2)
            .unwrap()
            .finish(&issuer_keypair)
            .now_or_never()
            .unwrap()
            .unwrap()
            .into_verified();

        let payload = CredentialPayload::from_sd_jwt(sd_jwt.clone())
            .expect("creating and validating CredentialPayload from SD-JWT should succeed");

        assert_eq!(payload.previewable_payload.attestation_type, sd_jwt.claims().vct);
    }

    #[test]
    fn test_from_sd_jwt_error_missing_status() {
        let holder_key = SigningKey::generate();

        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_keypair = ca.generate_issuer_mock().unwrap();

        let claims = SdJwtVcClaims {
            status: None,
            ..SdJwtVcClaims::example_from_json(holder_key.verifying_key(), json!({}), &MockTimeGenerator::default())
        };

        let sd_jwt = SdJwtBuilder::new(claims)
            .finish(&issuer_keypair)
            .now_or_never()
            .unwrap()
            .unwrap()
            .into_verified();

        let error = CredentialPayload::from_sd_jwt(sd_jwt)
            .expect_err("creating CredentialPayload from an SD-JWT without a status should fail");

        assert_matches!(error, CredentialPayloadFromSdJwtError::MissingStatusClaim);
    }

    #[test]
    fn test_to_sd_jwt() {
        let time_generator = MockTimeGenerator::default();

        let holder_key = MockRemoteEcdsaKey::new_random("holder_key".to_string());
        let wscd = MockRemoteWscd::new(vec![holder_key.clone()]);

        let ca = Ca::generate_mock();
        let issuer_key_pair = generate_issuer_mock_with_registration(&ca, &IssuerRegistration::new_mock()).unwrap();

        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::example_with_claim_name(
            PID_ATTESTATION_TYPE,
            "family_name",
        ));

        let credential_payload = CredentialPayload::example_with_attributes(
            PID_ATTESTATION_TYPE,
            Attributes::example([(["family_name"], Attribute::Text(String::from("De Bruijn")))]),
            holder_key.verifying_key(),
            &time_generator,
        );

        let sd_jwt = credential_payload
            .into_signed_sd_jwt(&metadata, &issuer_key_pair)
            .now_or_never()
            .unwrap()
            .unwrap();

        let (presented_sd_jwts, _poa) = UnsignedSdJwtPresentation::sign_multiple(
            vec_nonempty![(
                sd_jwt.into_verified().into_presentation_builder().finish(),
                "holder_key"
            )],
            KeyBindingJwtBuilder::new(
                String::from("https://aud.example.com"),
                Nonce::from(String::from("nonce123")),
            ),
            &wscd,
            (),
            &MockTimeGenerator::default(),
        )
        .now_or_never()
        .unwrap()
        .expect("signing a single SdJwtPresentation using the WSCD should succeed");

        let kb_verification_options = KbVerificationOptions {
            expected_aud: "https://aud.example.com",
            expected_nonce: &Nonce::from(String::from("nonce123")),
            iat_leeway: Duration::from_secs(5),
            iat_acceptance_window: Duration::from_secs(60),
        };

        let presented_sd_jwt = presented_sd_jwts.into_iter().exactly_one().unwrap().into_unverified();
        presented_sd_jwt
            .into_verified_against_trust_anchors(
                &TrustAnchors::from(&ca),
                &kb_verification_options,
                &time_generator,
                &RevocationVerifier::new_without_caching(Arc::new(StatusListClientStub::new(issuer_key_pair))),
            )
            .now_or_never()
            .unwrap()
            .unwrap();
    }

    #[test]
    fn test_matches_existing() {
        let epoch_generator = MockTimeGenerator::epoch();

        let mut new = PreviewableCredentialPayload {
            attestation_type: String::from("att_type_1"),
            expires: Some(Utc.with_ymd_and_hms(2000, 1, 1, 0, 1, 1).unwrap().into()),
            not_before: Some(Utc.with_ymd_and_hms(1969, 1, 1, 0, 1, 1).unwrap().into()),
            attributes: IndexMap::from([(String::from("attr1"), Attribute::Text(String::from("val1")))]).into(),
        };

        let mut existing = new.clone();
        assert!(new.matches_existing(&existing, &epoch_generator));

        existing.attestation_type = String::from("att_type_2");
        assert!(!new.matches_existing(&existing, &epoch_generator));

        let mut existing = new.clone();
        existing.attributes = IndexMap::from([(String::from("attr1"), Attribute::Text(String::from("val2")))]).into();
        assert!(!new.matches_existing(&existing, &epoch_generator));

        let mut existing = new.clone();
        existing.not_before = Some(Utc.with_ymd_and_hms(1970, 1, 1, 0, 1, 1).unwrap().into());
        assert!(
            new.matches_existing(&existing, &epoch_generator),
            "the payloads should match if the nbf of the new payload is in the past and the rest is the same"
        );

        let existing = new.clone();
        new.not_before = Some(Utc.with_ymd_and_hms(1980, 1, 1, 0, 1, 1).unwrap().into());
        assert!(
            !new.matches_existing(&existing, &epoch_generator),
            "the payloads should not match if the nbf of the new payload is in the future and different from the \
             existing payload"
        );
    }
}
