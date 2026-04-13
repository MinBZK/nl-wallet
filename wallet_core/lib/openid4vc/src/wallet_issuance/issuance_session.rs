use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::convert::identity;

use derive_more::Debug;
use futures::TryFutureExt;
use futures::future::try_join_all;
use itertools::Itertools;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use rand_core::OsRng;
use reqwest::Method;
use reqwest::Response;
use reqwest::header::AUTHORIZATION;
use reqwest::header::ToStrError;
use rustls_pki_types::TrustAnchor;
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;

use attestation_data::attributes::AttributesTraversalBehaviour;
use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::credential_payload::CredentialPayload;
use attestation_types::claim_path::ClaimPath;
use crypto::x509::BorrowingCertificate;
use http_utils::reqwest::HttpJsonClient;
use jwt::wua::WuaDisclosure;
use mdoc::ATTR_RANDOM_LENGTH;
use mdoc::holder::Mdoc;
use mdoc::utils::serialization::TaggedBytes;
use sd_jwt::error::DecoderError;
use sd_jwt::sd_jwt::VerifiedSdJwt;
use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use utils::generator::TimeGenerator;
use utils::single_unique::SingleUnique;
use utils::vec_at_least::VecNonEmpty;
use wscd::Poa;
use wscd::wscd::IssuanceWscd;

use crate::CredentialErrorCode;
use crate::CredentialPreviewErrorCode;
use crate::ErrorResponse;
use crate::TokenErrorCode;
use crate::credential::Credential;
use crate::credential::CredentialRequest;
use crate::credential::CredentialRequestProof;
use crate::credential::CredentialRequestType;
use crate::credential::CredentialRequests;
use crate::credential::CredentialResponse;
use crate::credential::CredentialResponses;
use crate::dpop::DPOP_HEADER_NAME;
use crate::dpop::DPOP_NONCE_HEADER_NAME;
use crate::dpop::Dpop;
use crate::metadata::issuer_metadata::IssuerMetadata;
use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
use crate::nonce::response::NonceResponse;
use crate::preview::CredentialPreviewRequest;
use crate::preview::CredentialPreviewResponse;
use crate::token::AccessToken;
use crate::token::CredentialPreview;
use crate::token::TokenRequest;
use crate::token::TokenResponse;
use crate::wallet_issuance::IssuanceSession;
use crate::wallet_issuance::WalletIssuanceError;
use crate::wallet_issuance::credential::CredentialWithMetadata;
use crate::wallet_issuance::credential::IssuedCredential;
use crate::wallet_issuance::credential::IssuedCredentialCopies;
use crate::wallet_issuance::preview::NormalizedCredentialPreview;

#[derive(Debug)]
pub struct HttpIssuanceSession<H = HttpVcMessageClient> {
    message_client: H,
    session_state: IssuanceState,
}

/// Contract for sending OpenID4VCI protocol messages.
#[cfg_attr(test, mockall::automock)]
pub trait VcMessageClient {
    async fn request_token(
        &self,
        url: &Url,
        token_request: &TokenRequest,
        dpop_header: &Dpop,
    ) -> Result<(TokenResponse, Option<String>), WalletIssuanceError>;

    async fn request_credential_preview(
        &self,
        url: &Url,
        preview_request: &CredentialPreviewRequest,
        access_token: &AccessToken,
    ) -> Result<CredentialPreviewResponse, WalletIssuanceError>;

    async fn request_nonce(&self, url: Url) -> Result<(NonceResponse, Option<String>), WalletIssuanceError>;

    async fn request_credential(
        &self,
        url: &Url,
        credential_request: &CredentialRequest,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<CredentialResponse, WalletIssuanceError>;

    async fn request_credentials(
        &self,
        url: &Url,
        credential_requests: &CredentialRequests,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<CredentialResponses, WalletIssuanceError>;

    async fn reject(&self, url: &Url, dpop_header: &str, access_token_header: &str) -> Result<(), WalletIssuanceError>;
}

#[derive(Debug)]
pub struct HttpVcMessageClient {
    http_client: HttpJsonClient,
}

impl HttpVcMessageClient {
    pub fn new(http_client: HttpJsonClient) -> Self {
        Self { http_client }
    }

    fn dpop_nonce(response: &Response) -> Result<Option<String>, ToStrError> {
        let dpop_nonce = response
            .headers()
            .get(DPOP_NONCE_HEADER_NAME)
            .map(|val| val.to_str())
            .transpose()?
            .map(str::to_string);

        Ok(dpop_nonce)
    }
}

impl VcMessageClient for HttpVcMessageClient {
    async fn request_token(
        &self,
        url: &Url,
        token_request: &TokenRequest,
        dpop_header: &Dpop,
    ) -> Result<(TokenResponse, Option<String>), WalletIssuanceError> {
        self.http_client
            .post(url.as_ref(), |builder| {
                builder
                    .header(DPOP_HEADER_NAME, dpop_header.to_string())
                    .form(token_request)
            })
            .map_err(WalletIssuanceError::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<TokenErrorCode>>().await?;
                    Err(WalletIssuanceError::TokenRequest(Box::new(error)))
                } else {
                    let dpop_nonce = Self::dpop_nonce(&response)?;
                    let deserialized = response.json::<TokenResponse>().await?;
                    Ok((deserialized, dpop_nonce))
                }
            })
            .await
    }

    async fn request_credential_preview(
        &self,
        url: &Url,
        preview_request: &CredentialPreviewRequest,
        access_token: &AccessToken,
    ) -> Result<CredentialPreviewResponse, WalletIssuanceError> {
        self.http_client
            .post(url.as_ref(), |builder| {
                builder.bearer_auth(access_token.as_ref()).json(preview_request)
            })
            .map_err(WalletIssuanceError::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<CredentialPreviewErrorCode>>().await?;
                    Err(WalletIssuanceError::CredentialPreviewRequest(Box::new(error)))
                } else {
                    let response = response.json().await?;
                    Ok(response)
                }
            })
            .await
    }

    async fn request_nonce(&self, url: Url) -> Result<(NonceResponse, Option<String>), WalletIssuanceError> {
        let response = self.http_client.post(url, identity).await?.error_for_status()?;

        let dpop_nonce = Self::dpop_nonce(&response)?;
        let nonce_response = response.json().await?;

        Ok((nonce_response, dpop_nonce))
    }

    async fn request_credential(
        &self,
        url: &Url,
        credential_request: &CredentialRequest,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<CredentialResponse, WalletIssuanceError> {
        self.request(url, credential_request, dpop_header, access_token_header)
            .await
    }

    async fn request_credentials(
        &self,
        url: &Url,
        credential_requests: &CredentialRequests,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<CredentialResponses, WalletIssuanceError> {
        self.request(url, credential_requests, dpop_header, access_token_header)
            .await
    }

    async fn reject(&self, url: &Url, dpop_header: &str, access_token_header: &str) -> Result<(), WalletIssuanceError> {
        self.http_client
            .delete(url.as_ref(), |builder| {
                builder
                    .header(DPOP_HEADER_NAME, dpop_header)
                    .header(AUTHORIZATION, access_token_header)
            })
            .map_err(WalletIssuanceError::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<CredentialErrorCode>>().await?;
                    Err(WalletIssuanceError::CredentialRequest(Box::new(error)))
                } else {
                    Ok(())
                }
            })
            .await?;
        Ok(())
    }
}

impl HttpVcMessageClient {
    async fn request<T: Serialize, S: DeserializeOwned>(
        &self,
        url: &Url,
        request: &T,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<S, WalletIssuanceError> {
        self.http_client
            .post(url.as_ref(), |builder| {
                builder
                    .header(DPOP_HEADER_NAME, dpop_header)
                    .header(AUTHORIZATION, access_token_header)
                    .json(request)
            })
            .map_err(WalletIssuanceError::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<CredentialErrorCode>>().await?;
                    Err(WalletIssuanceError::CredentialRequest(Box::new(error)))
                } else {
                    let response = response.json().await?;
                    Ok(response)
                }
            })
            .await
    }
}

#[cfg_attr(test, derive(Clone))]
#[derive(Debug)]
struct IssuanceState {
    access_token: AccessToken,
    normalized_credential_previews: VecNonEmpty<NormalizedCredentialPreview>,
    credential_request_types: VecNonEmpty<CredentialRequestType>,
    issuer_registration: IssuerRegistration,
    issuer_metadata: IssuerMetadata,
    #[debug(skip)]
    dpop_signing_key: SigningKey,
    dpop_nonce: Option<String>,
}

fn credential_request_types_from_preview(
    normalized_credential_previews: &VecNonEmpty<NormalizedCredentialPreview>,
) -> Result<VecNonEmpty<CredentialRequestType>, WalletIssuanceError> {
    // The OpenID4VCI `/batch_credential` endpoints supports issuance of multiple attestations, but the protocol
    // has no support (yet) for issuance of multiple copies of multiple attestations.
    // We implement this below by simply flattening the relevant nested iterators when communicating with the
    // issuer.
    //
    // The `/batch_credential` endpoint also does not support reading the `CredentialRequest::credential_type`
    // field, it will simply provide the wallet with copies of all of the credential formats it proposes.
    // For this reason, it is simply an error if the wallet does not support all of the formats proposed by
    // the issuer.
    //
    // TODO (PVW-4366): Have the batch issuance endpoint consider the `credential_type` field
    //                  of the `CredentialRequest`s and only issue those formats.

    let credential_request_types = normalized_credential_previews
        .iter()
        .map(|preview| {
            let mut unsupported_formats = HashSet::new();

            // Construct a `Vec<CredentialRequestType>`, with one entry
            // per copy per supported format for this credential.
            let request_types = preview
                .content
                .copies_per_format
                .iter()
                .flat_map(|(format, copies)| {
                    let request_type = CredentialRequestType::from_format(
                        *format,
                        preview.content.credential_payload.attestation_type.clone(),
                    );

                    if request_type.is_none() {
                        unsupported_formats.insert(*format);
                    }

                    request_type.map(|request_type| itertools::repeat_n(request_type, copies.get().into()))
                })
                .flatten()
                .collect_vec();

            // If we do not support one of the proposed formats this constitutes an error, as described above.
            if !unsupported_formats.is_empty() {
                return Err(WalletIssuanceError::UnsupportedCredentialFormat(
                    preview.content.credential_payload.attestation_type.clone(),
                    unsupported_formats,
                ));
            }

            Ok(request_types)
        })
        .process_results(|iter| iter.flatten().collect_vec())?
        .try_into()
        .unwrap(); // we're iterating over a VecNonEmpty

    Ok(credential_request_types)
}

impl<H: VcMessageClient> HttpIssuanceSession<H> {
    pub(crate) async fn create(
        message_client: H,
        issuer_metadata: IssuerMetadata,
        oauth_metadata: AuthorizationServerMetadata,
        token_request: TokenRequest,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Self, WalletIssuanceError> {
        let credential_preview_endpoint = issuer_metadata
            .credential_preview_endpoint
            .as_ref()
            .map(|url| url.clone().into_url())
            .ok_or(WalletIssuanceError::NoCredentialPreviewEndpoint)?; // TODO (PVW-5559): skip preview when no credential preview endpoint

        // TODO: Get the credential configuration ids from the `CredentialOffer` instead.
        let credential_configuration_ids: VecNonEmpty<String> = issuer_metadata
            .credential_configurations_supported
            .keys()
            .cloned()
            .collect_vec()
            .try_into()
            .map_err(|_| WalletIssuanceError::NoCredentialConfigurationsSupported)?;

        // According to HAIP, if the issuer requires key binding for any of its credential configurations, it MUST also
        // offer a nonce endpoint. As the wallet, we interpret this a bit more loosely and reject issuance whenever any
        // of the credential configurations offered require key binding, as the metadata may contain other
        // configurations that do not concern this particular issuance session.
        // See: https://openid.net/specs/openid4vc-high-assurance-interoperability-profile-1_0.html#section-4.1-5
        if issuer_metadata.nonce_endpoint.is_none()
            && credential_configuration_ids.iter().any(|config_id| {
                issuer_metadata
                    .credential_configurations_supported
                    .get(config_id)
                    // This unwrap is safe, because we just got the ids from the issuer metadata.
                    .unwrap()
                    .cryptographic_binding
                    .is_some()
            })
        {
            return Err(WalletIssuanceError::NoNonceEndpoint);
        }

        let token_endpoint = oauth_metadata.token_endpoint.clone();
        let dpop_signing_key = SigningKey::random(&mut OsRng);
        let dpop_header = Dpop::new(&dpop_signing_key, token_endpoint.clone(), &Method::POST, None, None)?;

        let (token_response, dpop_nonce) = message_client
            .request_token(&token_endpoint, &token_request, &dpop_header)
            .await?;

        // Call the credential preview endpoint to retrieve the credential previews.
        let preview_request = CredentialPreviewRequest::CredentialConfigurationIds {
            credential_configuration_ids,
        };
        let preview_response = message_client
            .request_credential_preview(
                &credential_preview_endpoint,
                &preview_request,
                &token_response.access_token,
            )
            .await?;

        let credential_previews: VecNonEmpty<CredentialPreview> = preview_response.credential_previews;

        let issuer_registration = credential_previews
            .iter()
            .map(|preview| preview.issuer_registration())
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .single_unique()
            .map_err(WalletIssuanceError::DifferentIssuerRegistrations)?
            .expect("there are always credential_previews in the preview response")
            .clone();

        let normalized_credential_previews: VecNonEmpty<_> = credential_previews
            .into_iter()
            .map(|preview| {
                // Verify the issuer certificate against the trust anchors.
                preview.verify(trust_anchors)?;
                let state = NormalizedCredentialPreview::try_new(preview)?;
                Ok::<_, WalletIssuanceError>(state)
            })
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .unwrap(); // credential_previews is VecNonEmpty

        let credential_request_types = credential_request_types_from_preview(&normalized_credential_previews)?;

        let session_state = IssuanceState {
            access_token: token_response.access_token,
            normalized_credential_previews,
            credential_request_types,
            issuer_registration,
            issuer_metadata,
            dpop_signing_key,
            dpop_nonce,
        };

        let issuance_client = Self {
            message_client,
            session_state,
        };

        Ok(issuance_client)
    }
}

impl<H: VcMessageClient> IssuanceSession for HttpIssuanceSession<H> {
    async fn accept_issuance<W>(
        &mut self,
        trust_anchors: &[TrustAnchor<'_>],
        wscd: &W,
        include_wua: bool,
    ) -> Result<Vec<CredentialWithMetadata>, WalletIssuanceError>
    where
        W: IssuanceWscd<Poa = Poa>,
    {
        let issuer_metadata = &self.session_state.issuer_metadata;
        let key_count = self.session_state.credential_request_types.len();

        // Determine the correct credential endpoint URL, to be used below.
        let credential_endpoint_url = if key_count.get() == 1 {
            &issuer_metadata.credential_endpoint
        } else {
            issuer_metadata
                .batch_credential_endpoint
                .as_ref()
                .ok_or(WalletIssuanceError::NoBatchCredentialEndpoint)?
        }
        .as_url();

        // Fetch one nonce from the nonce endpoint, if defined in the issuer metadata.
        let c_nonce = match issuer_metadata.nonce_endpoint.as_ref() {
            None => None,
            Some(nonce_endpoint) => {
                let (NonceResponse { c_nonce }, dpop_nonce) = self
                    .message_client
                    .request_nonce(nonce_endpoint.clone().into_url())
                    .await?;

                // If the nonce endpoint response included a DPoP-Nonce header, update the value in the state.
                if let Some(dpop_nonce) = dpop_nonce {
                    self.session_state.dpop_nonce = Some(dpop_nonce);
                }

                Some(c_nonce)
            }
        };

        let mut issuance_data = wscd
            .perform_issuance(
                key_count,
                self.session_state
                    .issuer_metadata
                    .credential_issuer
                    .as_ref()
                    .to_string(),
                c_nonce,
                include_wua,
            )
            .await
            .map_err(|e| WalletIssuanceError::PrivateKeyGeneration(e.into()))?;

        let proofs = issuance_data
            .pops
            .into_iter()
            .map(|jwt| CredentialRequestProof::Jwt { jwt });

        // Call the amount of proofs we received N, which equals `key_count`.
        // Combining these with the key identifiers and attestation types, compute N public keys and
        // N credential requests.
        let (pubkeys, mut credential_requests): (Vec<_>, Vec<_>) = try_join_all(
            proofs
                .zip(issuance_data.key_identifiers.into_inner())
                .zip(self.session_state.credential_request_types.clone())
                .map(|((proof, id), credential_request_type)| async move {
                    let CredentialRequestProof::Jwt { jwt } = &proof;

                    // We assume here the WP gave us valid JWTs, and leave it up to the issuer to verify these.
                    let header = jwt.dangerous_parse_header_unverified()?;

                    let pubkey = header
                        .verifying_key()
                        .map_err(|e| WalletIssuanceError::VerifyingKeyFromPrivateKey(e.into()))?;
                    let cred_request = CredentialRequest {
                        credential_type: credential_request_type.into(),
                        proof: Some(proof),
                        attestations: None, // We set this field below if necessary
                        poa: None,          // We set this field below if necessary
                    };

                    Ok::<_, WalletIssuanceError>(((pubkey, id), cred_request))
                }),
        )
        .await?
        .into_iter()
        .unzip();

        // The following two unwraps are safe because N > 0, see above.
        let responses = match credential_requests.len() {
            1 => {
                let mut credential_request = credential_requests.pop().unwrap();
                credential_request.attestations = issuance_data.wua.take();
                credential_request.poa = issuance_data.poa.take();
                vec![
                    self.request_credential(credential_endpoint_url, &credential_request)
                        .await?,
                ]
            }
            _ => {
                let credential_requests = VecNonEmpty::try_from(credential_requests).unwrap();
                self.request_batch_credentials(
                    credential_endpoint_url,
                    credential_requests,
                    issuance_data.wua.take(),
                    issuance_data.poa.take(),
                )
                .await?
            }
        };
        let mut responses_and_pubkeys: VecDeque<_> = responses.into_iter().zip(pubkeys).collect();

        let docs = self
            .session_state
            .normalized_credential_previews
            .iter()
            .map(|preview| {
                let copies =
                    preview
                        .content
                        .copies_per_format
                        .iter()
                        .try_fold(vec![], |mut acc, (format, copies)| {
                            let copy_count: usize = copies.get().into();

                            // Consume the amount of copies from the front of `responses_and_keys`.
                            let mut cred_copies = responses_and_pubkeys
                                .drain(..copy_count)
                                .map(|(cred_response, (pubkey, key_id))| {
                                    let credential = cred_response
                                        .into_immediate_credential()
                                        .ok_or(WalletIssuanceError::DeferredIssuanceUnsupported)?;

                                    if credential.format() != *format {
                                        return Err(WalletIssuanceError::UnexpectedCredentialResponseType {
                                            expected: format.to_string(),
                                            actual: credential.clone(),
                                        });
                                    }

                                    // Convert the response into a credential, verifying it against both the
                                    // trust anchors and the credential preview we received in the preview.
                                    credential.into_issued_credential(key_id, &pubkey, preview, trust_anchors)
                                })
                                .collect::<Result<Vec<IssuedCredential>, _>>()?;

                            acc.append(&mut cred_copies);

                            Ok::<_, WalletIssuanceError>(acc)
                        })?;

                // Verify that each of the resulting mdocs contain exactly the same metadata integrity digest.
                let integrity = copies
                    .iter()
                    .map(|cred_copy| match cred_copy {
                        IssuedCredential::MsoMdoc { mdoc } => {
                            mdoc.type_metadata_integrity().map_err(WalletIssuanceError::Metadata)
                        }
                        IssuedCredential::SdJwt { sd_jwt, .. } => sd_jwt
                            .claims()
                            .vct_integrity
                            .as_ref()
                            .ok_or(WalletIssuanceError::MetadataIntegrityMissing),
                    })
                    .process_results(|iter| {
                        iter.unique()
                            .exactly_one()
                            .map_err(|_| WalletIssuanceError::MetadataIntegrityInconsistent)
                    })??;

                // Check that the integrity hash received in the credential matches
                // that of encoded JSON of the first metadata document.
                let verified_metadata = preview.raw_metadata.clone().into_verified(integrity.clone())?;

                Ok::<_, WalletIssuanceError>(CredentialWithMetadata::new(
                    IssuedCredentialCopies::new(
                        copies
                            .try_into()
                            .expect("the resulting vector is never empty since 'copies' is nonzero"),
                    ),
                    preview.content.credential_payload.attestation_type.clone(),
                    preview.content.credential_payload.expires,
                    preview.content.credential_payload.not_before,
                    preview.normalized_metadata.extended_vcts(),
                    verified_metadata,
                ))
            })
            .try_collect()?;

        Ok(docs)
    }

    async fn reject_issuance(self) -> Result<(), WalletIssuanceError> {
        let url = self
            .session_state
            .issuer_metadata
            .batch_credential_endpoint
            .as_ref()
            .ok_or(WalletIssuanceError::NoBatchCredentialEndpoint)?
            .as_url();

        let (dpop_header, access_token_header) = self.session_state.auth_headers(url.clone(), &Method::DELETE)?;

        self.message_client
            .reject(url, &dpop_header, &access_token_header)
            .await?;

        Ok(())
    }

    fn normalized_credential_preview(&self) -> &[NormalizedCredentialPreview] {
        self.session_state.normalized_credential_previews.as_ref()
    }

    fn issuer_registration(&self) -> &IssuerRegistration {
        &self.session_state.issuer_registration
    }
}

impl<H: VcMessageClient> HttpIssuanceSession<H> {
    async fn request_credential(
        &self,
        url: &Url,
        credential_request: &CredentialRequest,
    ) -> Result<CredentialResponse, WalletIssuanceError> {
        // let url = self
        //     .session_state
        //     .issuer_metadata
        //     .credential_endpoint
        //     .clone()
        //     .into_url();

        let (dpop_header, access_token_header) = self.session_state.auth_headers(url.clone(), &Method::POST)?;

        let response = self
            .message_client
            .request_credential(url, credential_request, &dpop_header, &access_token_header)
            .await?;

        Ok(response)
    }

    async fn request_batch_credentials(
        &self,
        url: &Url,
        credential_requests: VecNonEmpty<CredentialRequest>,
        wua_disclosure: Option<WuaDisclosure>,
        poa: Option<Poa>,
    ) -> Result<Vec<CredentialResponse>, WalletIssuanceError> {
        // let url = self
        //     .session_state
        //     .issuer_metadata
        //     .batch_credential_endpoint
        //     .clone()
        //     .map(|u| u.into_url())
        //     .ok_or(WalletIssuanceError::NoBatchCredentialEndpoint)?;

        let (dpop_header, access_token_header) = self.session_state.auth_headers(url.clone(), &Method::POST)?;

        let expected_response_count = credential_requests.len().get();
        let responses = self
            .message_client
            .request_credentials(
                url,
                &CredentialRequests {
                    credential_requests,
                    attestations: wua_disclosure,
                    poa,
                },
                &dpop_header,
                &access_token_header,
            )
            .await?;

        // The server must have responded with enough credential responses, N, so that the caller has exactly enough
        // responses for all copies of all credentials constructed.
        if responses.credential_responses.len() != expected_response_count {
            return Err(WalletIssuanceError::UnexpectedCredentialResponseCount {
                found: responses.credential_responses.len(),
                expected: expected_response_count,
            });
        }

        Ok(responses.credential_responses)
    }
}

impl Credential {
    /// Create a credential out of the credential response. Also verifies the credential.
    fn into_issued_credential(
        self,
        key_identifier: String,
        verifying_key: &VerifyingKey,
        preview: &NormalizedCredentialPreview,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<IssuedCredential, WalletIssuanceError> {
        match self {
            Self::MsoMdoc {
                credential: issuer_signed,
            } => {
                // Calculate the minimum of all the lengths of the random bytes
                // included in the attributes of `IssuerSigned`. If this value
                // is too low, we should not accept the attributes.
                if let Some(min) = issuer_signed.name_spaces.as_ref().and_then(|namespaces| {
                    namespaces
                        .as_ref()
                        .values()
                        .flat_map(|attributes| attributes.as_ref().iter().map(|TaggedBytes(item)| item.random.len()))
                        .min()
                }) && min < ATTR_RANDOM_LENGTH
                {
                    return Err(WalletIssuanceError::AttributeRandomLength(min, ATTR_RANDOM_LENGTH));
                }

                let credential_issuer_certificate = &issuer_signed
                    .issuer_auth
                    .signing_cert()
                    .map_err(WalletIssuanceError::IssuerCertificate)?;

                // Construct the new mdoc; this also verifies it against the trust anchors.
                let mdoc = Mdoc::new(key_identifier, *issuer_signed, &TimeGenerator, trust_anchors)
                    .map_err(WalletIssuanceError::MdocVerification)?;

                let issued_credential_payload =
                    CredentialPayload::from_mdoc(mdoc.clone(), &preview.normalized_metadata)?;

                Self::validate_credential(
                    preview,
                    verifying_key,
                    issued_credential_payload,
                    credential_issuer_certificate,
                )?;

                Ok(IssuedCredential::MsoMdoc { mdoc })
            }
            Self::SdJwt {
                credential: unverified_sd_jwt,
            } => {
                let sd_jwt = unverified_sd_jwt.into_verified_against_trust_anchors(trust_anchors, &TimeGenerator)?;
                let issued_credential_payload =
                    CredentialPayload::from_sd_jwt(sd_jwt.clone(), &preview.normalized_metadata)?;

                // Store claim paths to later use in validation of selective disclosability of claims.
                // This prevents cloning `issued_credential_payload`.
                let issued_claims = issued_credential_payload
                    .previewable_payload
                    .attributes
                    .claim_paths(AttributesTraversalBehaviour::OnlyLeaves);

                Self::validate_credential(
                    preview,
                    verifying_key,
                    issued_credential_payload,
                    sd_jwt.issuer_certificate(),
                )?;

                // Verify whether each claims selective disclosability matches the metadata.
                // This validation is SD-JWT specific, and therefore cannot be part of `validate_credential`.
                Self::verify_selective_disclosability(&sd_jwt, issued_claims, preview.normalized_metadata.clone())?;

                Ok(IssuedCredential::SdJwt { key_identifier, sd_jwt })
            }
        }
    }

    fn validate_credential(
        preview: &NormalizedCredentialPreview,
        holder_pubkey: &VerifyingKey,
        credential_payload: CredentialPayload,
        credential_issuer_certificate: &BorrowingCertificate,
    ) -> Result<(), WalletIssuanceError> {
        let NormalizedCredentialPreview { content, .. } = preview;

        if credential_payload.confirmation_key.verifying_key()? != *holder_pubkey {
            return Err(WalletIssuanceError::PublicKeyMismatch);
        }

        // The issuer certificate inside the mdoc has to equal the one that the issuer previously announced
        // in the credential preview.
        if credential_issuer_certificate != &content.issuer_certificate {
            return Err(WalletIssuanceError::IssuerMismatch);
        }

        // Check that our mdoc contains exactly the attributes the issuer said it would have.
        // Note that this also means that the mdoc's attributes must match the received metadata,
        // as both the metadata and attributes are the same as when we checked this for the preview.
        if credential_payload.previewable_payload != content.credential_payload {
            return Err(WalletIssuanceError::IssuedCredentialMismatch {
                actual: Box::new(credential_payload.previewable_payload),
                expected: Box::new(content.credential_payload.clone()),
            });
        }

        Ok(())
    }

    fn verify_selective_disclosability(
        sd_jwt: &VerifiedSdJwt,
        issued_claims: Vec<VecNonEmpty<ClaimPath>>,
        metadata: NormalizedTypeMetadata,
    ) -> Result<(), WalletIssuanceError> {
        let sd_metadata = metadata
            .into_presentation_components()
            .2
            .into_iter()
            .map(|md| (md.path.into_inner(), md.sd))
            .collect();

        // Iterate over the issued_claims, validating each element in the path against the metadata.
        // This implementation will ignore any (optional) claims that do exist in the metadata but are not issued.
        // Validating whether all required claims are issued is done by `validate_credential`.
        // This will also prevent traversing and decoding the same disclosures several times for nested disclosures.
        for issued_claim in issued_claims {
            Self::verify_claim_selective_disclosability(sd_jwt, issued_claim.as_slice(), &sd_metadata)?;
        }

        Ok(())
    }

    fn verify_claim_selective_disclosability(
        sd_jwt: &VerifiedSdJwt,
        claim_to_verify: &[ClaimPath],
        sd_metadata: &HashMap<Vec<ClaimPath>, ClaimSelectiveDisclosureMetadata>,
    ) -> Result<(), WalletIssuanceError> {
        sd_jwt
            .verify_selective_disclosability(claim_to_verify, sd_metadata)
            .map_err(DecoderError::ClaimStructure)?;

        Ok(())
    }
}

impl IssuanceState {
    fn auth_headers(&self, url: Url, method: &Method) -> Result<(String, String), WalletIssuanceError> {
        let dpop_header = Dpop::new(
            &self.dpop_signing_key,
            url,
            method,
            Some(&self.access_token),
            self.dpop_nonce.clone(),
        )?;

        let access_token_header = "DPoP ".to_string() + self.access_token.as_ref();

        Ok((dpop_header.to_string(), access_token_header))
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;
    use std::sync::Arc;
    use std::time::Duration;
    use std::vec;

    use assert_matches::assert_matches;
    use chrono::Utc;
    use futures::FutureExt;
    use indexmap::IndexMap;
    use mockall::predicate::eq;
    use rstest::rstest;
    use serde_bytes::ByteBuf;
    use serde_json::json;
    use ssri::Integrity;

    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::attributes::Attributes;
    use attestation_data::auth::LocalizedStrings;
    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::credential_payload::PreviewableCredentialPayload;
    use attestation_data::x509::generate::mock::generate_pid_issuer_mock_with_registration;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use attestation_types::qualification::AttestationQualification;
    use attestation_types::status_claim::StatusClaim;
    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use crypto::x509::CertificateError;
    use jwt::jwk::jwk_to_p256;
    use jwt::nonce::Nonce;
    use mdoc::utils::serialization::TaggedBytes;
    use sd_jwt::builder::SignedSdJwt;
    use sd_jwt::claims::ClaimName;
    use sd_jwt::error::ClaimError;
    use sd_jwt::test::conceal_and_sign;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::TypeMetadata;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_nonempty;
    use wscd::mock_remote::MockRemoteWscd;

    use crate::Format;
    use crate::issuer_identifier::IssuerIdentifier;
    use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
    use crate::metadata::well_known::WellKnownMetadata;
    use crate::preview::CredentialPreviewResponse;
    use crate::token::CredentialPreview;
    use crate::token::CredentialPreviewContent;
    use crate::token::CredentialPreviewError;
    use crate::token::TokenResponse;
    use crate::wallet_issuance::TypeMetadataChainError;
    use crate::wallet_issuance::WalletIssuanceError;

    use super::*;

    fn test_start_issuance(
        ca: &Ca,
        trust_anchor: TrustAnchor,
        issuer_metadata: IssuerMetadata,
        preview_payloads: Vec<PreviewableCredentialPayload>,
        type_metadata: TypeMetadata,
        formats: Vec<Format>,
    ) -> Result<HttpIssuanceSession<MockVcMessageClient>, WalletIssuanceError> {
        let issuance_key = generate_pid_issuer_mock_with_registration(ca, IssuerRegistration::new_mock()).unwrap();

        let mut mock_msg_client = MockVcMessageClient::new();
        mock_msg_client
            .expect_request_token()
            .return_once(move |_url, _token_request, _dpop_header| {
                let token_response = TokenResponse::new("access_token".to_string().into());
                Ok((token_response, None))
            });
        mock_msg_client
            .expect_request_credential_preview()
            .return_once(move |_url, _request, _access_token| {
                let (_, _, type_metadata) = TypeMetadataDocuments::from_single_example(type_metadata);

                let previews = preview_payloads
                    .into_iter()
                    .map(|preview_payload| CredentialPreview {
                        content: CredentialPreviewContent {
                            copies_per_format: formats.iter().map(|format| (*format, NonZeroU8::MIN)).collect(),
                            credential_payload: preview_payload,
                            issuer_certificate: issuance_key.certificate().clone(),
                        },
                        type_metadata: type_metadata.clone(),
                    })
                    .collect_vec()
                    .try_into()
                    .unwrap();

                Ok(CredentialPreviewResponse {
                    credential_previews: previews,
                })
            });

        let oauth_metadata = AuthorizationServerMetadata::new_mock(issuer_metadata.issuer_identifier().clone());

        HttpIssuanceSession::create(
            mock_msg_client,
            issuer_metadata,
            oauth_metadata,
            TokenRequest::new_mock(),
            &[trust_anchor],
        )
        .now_or_never()
        .unwrap()
    }

    #[test]
    fn test_start_issuance_ok() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let session = test_start_issuance(
            &ca,
            ca.to_trust_anchor(),
            IssuerMetadata::new_mock("https://example.com".parse().unwrap(), PID_ATTESTATION_TYPE),
            vec![PreviewableCredentialPayload::example_family_name(
                &MockTimeGenerator::default(),
            )],
            TypeMetadata::pid_example(),
            vec![Format::MsoMdoc],
        )
        .expect("starting issuance session should succeed");

        let NormalizedCredentialPreview {
            content,
            normalized_metadata,
            ..
        } = &session.normalized_credential_preview()[0];

        assert_matches!(
                &content.credential_payload.attributes.as_ref()["family_name"],
                Attribute::Single(AttributeValue::Text(v)) if v == "De Bruijn");

        assert_eq!(
            *normalized_metadata,
            TypeMetadataDocuments::from_single_example(TypeMetadata::pid_example())
                .2
                .into_normalized(&content.credential_payload.attestation_type)
                .unwrap()
                .0
        );
    }

    #[test]
    fn test_start_issuance_no_nonce_endpoint() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        // Starting issuance when the issuer metadata indicates that key
        // binding is mandatary, yet offers no nonce endpoint should fail.
        let mut issuer_metadata =
            IssuerMetadata::new_mock("https://example.com".parse().unwrap(), PID_ATTESTATION_TYPE);
        issuer_metadata.nonce_endpoint = None;

        let error = test_start_issuance(
            &ca,
            ca.to_trust_anchor(),
            issuer_metadata.clone(),
            vec![PreviewableCredentialPayload::example_family_name(
                &MockTimeGenerator::default(),
            )],
            TypeMetadata::pid_example(),
            vec![Format::MsoMdoc],
        )
        .expect_err("starting issuance session should not succeed");

        assert_matches!(error, WalletIssuanceError::NoNonceEndpoint);

        // When key binding is not mandatory however, the nonce endpoint can be absent.
        for config in issuer_metadata.credential_configurations_supported.values_mut() {
            config.cryptographic_binding = None;
        }

        let _ = test_start_issuance(
            &ca,
            ca.to_trust_anchor(),
            issuer_metadata,
            vec![PreviewableCredentialPayload::example_family_name(
                &MockTimeGenerator::default(),
            )],
            TypeMetadata::pid_example(),
            vec![Format::MsoMdoc],
        )
        .expect("starting issuance session should succeed");
    }

    #[test]
    fn test_start_issuance_untrusted_credential_preview() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let other_ca = Ca::generate_issuer_mock_ca().unwrap();

        let error = test_start_issuance(
            &ca,
            other_ca.to_trust_anchor(),
            IssuerMetadata::new_mock("https://example.com".parse().unwrap(), PID_ATTESTATION_TYPE),
            vec![PreviewableCredentialPayload::example_family_name(
                &MockTimeGenerator::default(),
            )],
            TypeMetadata::pid_example(),
            vec![Format::MsoMdoc],
        )
        .expect_err("starting issuance session should not succeed");

        assert_matches!(
            error,
            WalletIssuanceError::CredentialPreview(CredentialPreviewError::Certificate(
                CertificateError::Verification(_)
            ))
        );
    }

    #[test]
    fn test_start_issuance_type_metadata_verification_error() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let error = test_start_issuance(
            &ca,
            ca.to_trust_anchor(),
            IssuerMetadata::new_mock("https://example.com".parse().unwrap(), PID_ATTESTATION_TYPE),
            vec![PreviewableCredentialPayload::example_empty(
                PID_ATTESTATION_TYPE,
                &MockTimeGenerator::default(),
            )],
            TypeMetadata::empty_example_with_attestation_type("other_attestation_type"),
            vec![Format::MsoMdoc],
        )
        .expect_err("starting issuance session should not succeed");

        assert_matches!(error, WalletIssuanceError::TypeMetadataVerification(_));
    }

    #[test]
    fn test_start_issuance_error_unsupported_credential_format() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let error = test_start_issuance(
            &ca,
            ca.to_trust_anchor(),
            IssuerMetadata::new_mock("https://example.com".parse().unwrap(), PID_ATTESTATION_TYPE),
            vec![PreviewableCredentialPayload::example_empty(
                PID_ATTESTATION_TYPE,
                &MockTimeGenerator::default(),
            )],
            TypeMetadata::pid_example(),
            vec![Format::AcVc, Format::MsoMdoc, Format::JwtVc],
        )
        .expect_err("starting issuance session should not succeed");

        assert_matches!(
            error,
            WalletIssuanceError::UnsupportedCredentialFormat(attestation_type, formats)
                if attestation_type == PID_ATTESTATION_TYPE && formats == HashSet::from([Format::JwtVc, Format::AcVc])
        );
    }

    #[test]
    fn test_start_issuance_error_different_issuer_registrations() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let issuance_key = generate_pid_issuer_mock_with_registration(&ca, IssuerRegistration::new_mock()).unwrap();
        let mut different_org = IssuerRegistration::new_mock();
        different_org.organization.display_name = LocalizedStrings::from(vec![("en", "different org name")]);
        let different_issuance_key = generate_pid_issuer_mock_with_registration(&ca, different_org).unwrap();

        let preview_payload =
            PreviewableCredentialPayload::example_empty(PID_ATTESTATION_TYPE, &MockTimeGenerator::default());
        let copies_per_format: IndexMap<Format, NonZeroU8> =
            IndexMap::from_iter([(Format::MsoMdoc, NonZeroU8::MIN), (Format::SdJwt, NonZeroU8::MIN)]);

        let mut mock_msg_client = MockVcMessageClient::new();
        mock_msg_client
            .expect_request_token()
            .return_once(move |_url, _token_request, _dpop_header| {
                let token_response = TokenResponse::new("access_token".to_string().into());
                Ok((token_response, None))
            });
        mock_msg_client
            .expect_request_credential_preview()
            .return_once(move |_url, _request, _access_token| {
                let (_, _, type_metadata) = TypeMetadataDocuments::from_single_example(TypeMetadata::pid_example());

                let previews = vec_nonempty![
                    CredentialPreview {
                        content: CredentialPreviewContent {
                            copies_per_format: copies_per_format.clone(),
                            credential_payload: preview_payload.clone(),
                            issuer_certificate: issuance_key.certificate().clone(),
                        },
                        type_metadata: type_metadata.clone(),
                    },
                    CredentialPreview {
                        content: CredentialPreviewContent {
                            copies_per_format,
                            credential_payload: preview_payload,
                            issuer_certificate: different_issuance_key.certificate().clone(),
                        },
                        type_metadata: type_metadata.clone(),
                    },
                ];

                Ok(CredentialPreviewResponse {
                    credential_previews: previews,
                })
            });

        let issuer_identifier: IssuerIdentifier = "https://issuer.example.com".parse().unwrap();
        let issuer_metadata = IssuerMetadata::new_mock(issuer_identifier.clone(), PID_ATTESTATION_TYPE);
        let oauth_metadata = AuthorizationServerMetadata::new_mock(issuer_identifier);

        let error = HttpIssuanceSession::create(
            mock_msg_client,
            issuer_metadata,
            oauth_metadata,
            TokenRequest::new_mock(),
            &[ca.to_trust_anchor()],
        )
        .now_or_never()
        .unwrap()
        .expect_err("starting issuance session should not succeed");

        assert_matches!(error, WalletIssuanceError::DifferentIssuerRegistrations(_));
    }

    /// Return a new session ready for `accept_issuance()`.
    fn new_session_state(
        normalized_credential_previews: VecNonEmpty<NormalizedCredentialPreview>,
        has_nonce_endpoint: bool,
    ) -> IssuanceState {
        let credential_request_types = credential_request_types_from_preview(&normalized_credential_previews).unwrap();
        let issuer_identifier = "https://issuer.example.com".parse().unwrap();

        let mut issuer_metadata = IssuerMetadata::new_mock(issuer_identifier, PID_ATTESTATION_TYPE);
        if !has_nonce_endpoint {
            issuer_metadata.nonce_endpoint = None;
        }

        IssuanceState {
            access_token: "access_token".to_string().into(),
            normalized_credential_previews,
            credential_request_types,
            issuer_registration: IssuerRegistration::new_mock(),
            issuer_metadata,
            dpop_signing_key: SigningKey::random(&mut OsRng),
            dpop_nonce: Some("dpop_nonce".to_string()),
        }
    }

    fn mock_openid_message_client_nonce(has_dpop_nonce: bool) -> MockVcMessageClient {
        let mut mock_msg_client = MockVcMessageClient::new();

        mock_msg_client
            .expect_request_nonce()
            .times(1)
            .with(eq(Url::parse("https://issuer.example.com/issuance/nonce").unwrap()))
            .return_once(move |_| {
                Ok((
                    NonceResponse {
                        c_nonce: Nonce::from("c_nonce".to_string()),
                    },
                    has_dpop_nonce.then(|| "new_dpop_nonce".to_string()),
                ))
            });

        mock_msg_client
    }

    #[derive(super::Debug, Clone)]
    struct MockCredentialSigner {
        pub trust_anchor: TrustAnchor<'static>,
        issuer_key: Arc<KeyPair>,
        metadata_integrity: Integrity,
        previewable_payload: PreviewableCredentialPayload,
        status: StatusClaim,
    }

    impl MockCredentialSigner {
        pub fn new_with_preview_state() -> (Self, NormalizedCredentialPreview) {
            let preview_payload = PreviewableCredentialPayload::example_family_name(&MockTimeGenerator::default());
            let type_metadata = TypeMetadata::example_with_claim_name(
                &preview_payload.attestation_type,
                "family_name",
                JsonSchemaPropertyType::String,
                None,
            );

            Self::from_metadata_and_payload_with_preview_data(type_metadata, preview_payload)
        }

        pub fn from_metadata_and_payload_with_preview_data(
            type_metadata: TypeMetadata,
            preview_payload: PreviewableCredentialPayload,
        ) -> (Self, NormalizedCredentialPreview) {
            let ca = Ca::generate_issuer_mock_ca().unwrap();
            let trust_anchor = ca.to_trust_anchor().to_owned();

            let issuer_registration = IssuerRegistration::new_mock();
            let issuer_key = generate_pid_issuer_mock_with_registration(&ca, issuer_registration.clone()).unwrap();
            let issuer_certificate = issuer_key.certificate().clone();

            let (attestation_type, metadata_integrity, metadata_documents) =
                TypeMetadataDocuments::from_single_example(type_metadata);
            let (normalized_metadata, raw_metadata) = metadata_documents.into_normalized(&attestation_type).unwrap();

            let signer = Self {
                trust_anchor,
                issuer_key: Arc::new(issuer_key),
                metadata_integrity,
                previewable_payload: preview_payload.clone(),
                status: StatusClaim::new_mock(),
            };

            let preview = NormalizedCredentialPreview {
                content: CredentialPreviewContent {
                    copies_per_format: IndexMap::from([(Format::MsoMdoc, NonZeroU8::MIN)]),
                    credential_payload: preview_payload,
                    issuer_certificate,
                },
                normalized_metadata,
                raw_metadata,
            };

            (signer, preview)
        }

        pub fn into_response_from_request(self, request: &CredentialRequest) -> CredentialResponse {
            let proof_jwt = match request.proof.as_ref().unwrap() {
                CredentialRequestProof::Jwt { jwt } => jwt,
            };
            let holder_pubkey = jwk_to_p256(&proof_jwt.dangerous_parse_header_unverified().unwrap().jwk).unwrap();

            self.into_response_from_holder_pubkey(&holder_pubkey)
        }

        pub fn into_response_from_holder_pubkey(self, holder_pubkey: &VerifyingKey) -> CredentialResponse {
            let credential_payload = CredentialPayload::from_previewable_credential_payload_unvalidated(
                self.previewable_payload,
                Utc::now(),
                holder_pubkey,
                self.metadata_integrity,
                self.status,
            )
            .unwrap();

            let (issuer_signed, _) = credential_payload
                .into_signed_mdoc(&self.issuer_key)
                .now_or_never()
                .unwrap()
                .unwrap();

            CredentialResponse::new_immediate(Credential::new_mdoc(issuer_signed))
        }
    }

    /// Check consistency and validity of the input of the /(batch_)credential endpoints.
    #[expect(clippy::too_many_arguments)]
    fn check_credential_endpoint_input(
        url: &Url,
        dpop_signing_key: &SigningKey,
        dpop_nonce: &str,
        dpop_header: &str,
        access_token_header: &str,
        attestations: &Option<WuaDisclosure>,
        use_wua: bool,
    ) {
        assert_eq!(access_token_header, "DPoP access_token".to_string());

        dpop_header
            .parse::<Dpop>()
            .unwrap()
            .verify_expecting_key(
                dpop_signing_key.verifying_key(),
                url,
                &Method::POST,
                Some(&"access_token".to_string().into()),
                Some(dpop_nonce),
            )
            .unwrap();

        if use_wua != attestations.is_some() {
            panic!("unexpected WUA usage");
        }
    }

    enum TestNonceEndpoint {
        Absent,
        Present,
        PresentWithDpopNonce,
    }

    #[rstest]
    fn test_accept_issuance(
        #[values(true, false)] use_wua: bool,
        #[values(true, false)] multiple_creds: bool,
        #[values(
            TestNonceEndpoint::Absent,
            TestNonceEndpoint::Present,
            TestNonceEndpoint::PresentWithDpopNonce
        )]
        nonce_endpoint: TestNonceEndpoint,
    ) {
        let (signer, preview_data) = MockCredentialSigner::new_with_preview_state();
        let trust_anchor = signer.trust_anchor.clone();
        let wscd = MockRemoteWscd::default();

        let (mut mock_msg_client, has_nonce_endpoint, expected_dpop_nonce) = match nonce_endpoint {
            TestNonceEndpoint::Absent => (MockVcMessageClient::new(), false, "dpop_nonce"),
            TestNonceEndpoint::Present => (mock_openid_message_client_nonce(false), true, "dpop_nonce"),
            TestNonceEndpoint::PresentWithDpopNonce => (mock_openid_message_client_nonce(true), true, "new_dpop_nonce"),
        };

        let session_state = new_session_state(
            if multiple_creds {
                vec_nonempty![preview_data.clone(), preview_data]
            } else {
                vec_nonempty![preview_data]
            },
            has_nonce_endpoint,
        );

        // The client must use `request_credentials()` (which uses `/batch_credentials`) iff more than one credential
        // is being issued, and `request_credential()` instead (which uses `/credential`).
        if multiple_creds {
            mock_msg_client.expect_request_credentials().times(1).return_once({
                let session_state = session_state.clone();
                move |url, credential_requests, dpop_header, access_token_header| {
                    check_credential_endpoint_input(
                        url,
                        &session_state.dpop_signing_key,
                        expected_dpop_nonce,
                        dpop_header,
                        access_token_header,
                        &credential_requests.attestations,
                        use_wua,
                    );

                    let credential_responses = credential_requests
                        .credential_requests
                        .iter()
                        .zip(itertools::repeat_n(
                            signer,
                            credential_requests.credential_requests.len().get(),
                        ))
                        .map(|(request, signer)| signer.into_response_from_request(request))
                        .collect();

                    Ok(CredentialResponses { credential_responses })
                }
            });
        } else {
            mock_msg_client.expect_request_credential().times(1).return_once({
                let session_state = session_state.clone();
                move |url, credential_request, dpop_header, access_token_header| {
                    check_credential_endpoint_input(
                        url,
                        &session_state.dpop_signing_key,
                        expected_dpop_nonce,
                        dpop_header,
                        access_token_header,
                        &credential_request.attestations,
                        use_wua,
                    );

                    let response = signer.into_response_from_request(credential_request);

                    Ok(response)
                }
            });
        }

        let credential_copies = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state,
        }
        .accept_issuance(&[trust_anchor], &wscd, use_wua)
        .now_or_never()
        .unwrap()
        .expect("accepting issuance should succeed");

        let expected_credential_count = if multiple_creds { 2 } else { 1 };
        assert_eq!(credential_copies.len(), expected_credential_count);
    }

    #[test]
    fn test_accept_issuance_wrong_response_count() {
        let (signer, preview_data) = MockCredentialSigner::new_with_preview_state();
        let trust_anchor = signer.trust_anchor.clone();

        let mut mock_msg_client = mock_openid_message_client_nonce(false);
        // let mut mock_msg_client = MockVcMessageClient::new();

        mock_msg_client.expect_request_credentials().return_once(
            |_url, credential_requests, _dpop_header, _access_token_header| {
                let response = signer.into_response_from_request(credential_requests.credential_requests.first());
                // Return one credential response.
                let responses = CredentialResponses {
                    credential_responses: vec![response],
                };

                Ok(responses)
            },
        );

        let error = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state: new_session_state(vec_nonempty![preview_data.clone(), preview_data], true),
        }
        .accept_issuance(&[trust_anchor], &MockRemoteWscd::default(), false)
        .now_or_never()
        .unwrap()
        .expect_err("accepting issuance should not succeed");

        assert_matches!(
            error,
            WalletIssuanceError::UnexpectedCredentialResponseCount { found: 1, expected: 2 }
        );
    }

    #[test]
    fn test_accept_issuance_credential_payload_error() {
        let (signer, preview_data) = MockCredentialSigner::from_metadata_and_payload_with_preview_data(
            TypeMetadata::example_with_claim_name(
                PID_ATTESTATION_TYPE,
                "family_name",
                JsonSchemaPropertyType::String,
                None,
            ),
            PreviewableCredentialPayload::example_with_attributes(
                PID_ATTESTATION_TYPE,
                Attributes::example([(["family_name"], AttributeValue::Integer(1))]),
                &MockTimeGenerator::default(),
            ),
        );
        let trust_anchor = signer.trust_anchor.clone();

        let session_state = new_session_state(vec_nonempty![preview_data], true);

        let mut mock_msg_client = mock_openid_message_client_nonce(false);
        // let mut mock_msg_client = MockVcMessageClient::new();

        mock_msg_client.expect_request_credential().times(1).return_once({
            move |_url, credential_request, _dpop_header, _access_token_header| {
                let response = signer.into_response_from_request(credential_request);

                Ok(response)
            }
        });

        let error = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state,
        }
        .accept_issuance(&[trust_anchor], &MockRemoteWscd::default(), false)
        .now_or_never()
        .unwrap()
        .expect_err("accepting issuance should not succeed");

        assert_matches!(error, WalletIssuanceError::MdocCredentialPayload(_));
    }

    #[test]
    fn test_accept_issuance_incorrect_resource_integrity() {
        let (mut signer, preview_data) = MockCredentialSigner::new_with_preview_state();
        let trust_anchor = signer.trust_anchor.clone();

        // Include a random resource integrity in the MSO of the returned mdoc.
        signer.metadata_integrity = Integrity::from(crypto::utils::random_bytes(32));

        let mut mock_msg_client = mock_openid_message_client_nonce(false);
        // let mut mock_msg_client = MockVcMessageClient::new();

        mock_msg_client.expect_request_credential().return_once(
            |_url, credential_request, _dpop_header, _access_token_header| {
                let response = signer.into_response_from_request(credential_request);

                Ok(response)
            },
        );

        let error = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state: new_session_state(vec_nonempty![preview_data], true),
        }
        .accept_issuance(&[trust_anchor], &MockRemoteWscd::default(), false)
        .now_or_never()
        .unwrap()
        .expect_err("accepting issuance should not succeed");

        assert_matches!(
            error,
            WalletIssuanceError::TypeMetadataVerification(TypeMetadataChainError::ResourceIntegrity(_))
        );
    }

    #[rstest]
    fn test_accept_issuance_deferred_issuance(#[values(false, true)] is_batch: bool) {
        let (signer, preview_data) = MockCredentialSigner::new_with_preview_state();
        let trust_anchor = signer.trust_anchor.clone();

        let mut mock_msg_client = mock_openid_message_client_nonce(false);
        // let mut mock_msg_client = MockVcMessageClient::new();

        let response = CredentialResponse::Deferred {
            transaction_id: "12345".to_string(),
            interval: Duration::from_hours(24),
        };

        let previews = if is_batch {
            mock_msg_client.expect_request_credentials().return_once(
                |_url, credential_requests, _dpop_header, _access_token_header| {
                    let responses = CredentialResponses {
                        credential_responses: vec![response; credential_requests.credential_requests.len().get()],
                    };

                    Ok(responses)
                },
            );

            vec_nonempty![preview_data.clone(), preview_data]
        } else {
            mock_msg_client
                .expect_request_credential()
                .return_once(|_url, _credential_request, _dpop_header, _access_token_header| Ok(response));

            vec_nonempty![preview_data]
        };

        let error = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state: new_session_state(previews, true),
        }
        .accept_issuance(&[trust_anchor], &MockRemoteWscd::default(), false)
        .now_or_never()
        .unwrap()
        .expect_err("accepting issuance should not succeed");

        assert_matches!(error, WalletIssuanceError::DeferredIssuanceUnsupported);
    }

    fn mock_credential_response_credential() -> (
        Credential,
        NormalizedCredentialPreview,
        VerifyingKey,
        TrustAnchor<'static>,
    ) {
        let (signer, preview_data) = MockCredentialSigner::new_with_preview_state();
        let trust_anchor = signer.trust_anchor.clone();
        let holder_pubkey = *SigningKey::random(&mut OsRng).verifying_key();
        let credential_response = signer
            .into_response_from_holder_pubkey(&holder_pubkey)
            .into_immediate_credential()
            .unwrap();

        (credential_response, preview_data, holder_pubkey, trust_anchor)
    }

    #[test]
    fn test_credential_response_into_mdoc() {
        let (credential, preview_data, holder_public_key, trust_anchor) = mock_credential_response_credential();

        let _issued_credential = credential
            .into_issued_credential("key_id".to_string(), &holder_public_key, &preview_data, &[trust_anchor])
            .expect("should be able to convert CredentialResponse into Mdoc");
    }

    #[test]
    fn test_credential_response_into_mdoc_public_key_mismatch_error() {
        let (credential, preview_data, _, trust_anchor) = mock_credential_response_credential();

        // Converting a `CredentialResponse` into an `Mdoc` using a different mdoc
        // public key than the one contained within the response should fail.
        let other_public_key = *SigningKey::random(&mut OsRng).verifying_key();
        let error = credential
            .into_issued_credential("key_id".to_string(), &other_public_key, &preview_data, &[trust_anchor])
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::PublicKeyMismatch);
    }

    #[test]
    fn test_credential_response_into_mdoc_attribute_random_length_error() {
        let (credential, preview_data, holder_public_key, trust_anchor) = mock_credential_response_credential();

        // Converting a `CredentialResponse` into an `Mdoc` from a response
        // that contains insufficient random data should fail.
        let credential = match credential {
            Credential::MsoMdoc {
                credential: mut issuer_signed,
            } => {
                let name_spaces = issuer_signed.name_spaces.as_mut().unwrap();

                name_spaces.modify_first_attributes(|attributes| {
                    let TaggedBytes(first_item) = attributes.first_mut().unwrap();

                    first_item.random = ByteBuf::from(b"12345");
                });

                Credential::new_mdoc(*issuer_signed)
            }
            Credential::SdJwt { .. } => panic!("unsupported credential request format"),
        };

        let error = credential
            .into_issued_credential("key_id".to_string(), &holder_public_key, &preview_data, &[trust_anchor])
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::AttributeRandomLength(5, ATTR_RANDOM_LENGTH));
    }

    #[test]
    fn test_credential_response_into_mdoc_issuer_certificate_mismatch_error() {
        let (credential, normalized_preview, holder_public_key, trust_anchor) = mock_credential_response_credential();

        // Converting a `CredentialResponse` into an `Mdoc` using a different issuer
        // public key in the preview than is contained within the response should fail.
        let other_ca = Ca::generate_issuer_mock_ca().unwrap();
        let other_issuance_key =
            generate_pid_issuer_mock_with_registration(&other_ca, IssuerRegistration::new_mock()).unwrap();
        let preview_data = NormalizedCredentialPreview {
            content: CredentialPreviewContent {
                issuer_certificate: other_issuance_key.certificate().clone(),
                ..normalized_preview.content
            },
            ..normalized_preview
        };

        let error = credential
            .into_issued_credential("key_id".to_string(), &holder_public_key, &preview_data, &[trust_anchor])
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::IssuerMismatch);
    }

    #[test]
    fn test_credential_response_into_mdoc_mdoc_verification_error() {
        let (credential, normalized_preview, holder_public_key, _) = mock_credential_response_credential();

        // Converting a `CredentialResponse` into an `Mdoc` that is
        // validated against incorrect trust anchors should fail.
        let error = credential
            .into_issued_credential("key_id".to_string(), &holder_public_key, &normalized_preview, &[])
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::MdocVerification(_));
    }

    #[test]
    fn test_credential_response_into_mdoc_issued_attributes_mismatch_error() {
        let (credential, mut normalized_preview, holder_public_key, trust_anchor) =
            mock_credential_response_credential();

        // Converting a `CredentialResponse` into an `Mdoc` with different attributes
        // in the preview than are contained within the response should fail.
        let attributes = PreviewableCredentialPayload::example_with_attributes(
            PID_ATTESTATION_TYPE,
            Attributes::example([
                (["new"], AttributeValue::Bool(true)),
                (["family_name"], AttributeValue::Text(String::from("De Bruijn"))),
            ]),
            &MockTimeGenerator::default(),
        )
        .attributes;
        normalized_preview.content.credential_payload.attributes = attributes;

        let error = credential
            .into_issued_credential(
                "key_id".to_string(),
                &holder_public_key,
                &normalized_preview,
                &[trust_anchor],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::IssuedCredentialMismatch { .. });
    }

    #[test]
    fn test_credential_response_into_mdoc_issued_issuer_mismatch_error() {
        let (credential, mut normalized_preview, holder_public_key, trust_anchor) =
            mock_credential_response_credential();

        // Converting a `CredentialResponse` into an `Mdoc` with a different `issuer_uri` in the preview than
        // contained within the response should fail.
        normalized_preview.content.credential_payload.issuer = "https://other-issuer.example.com".parse().unwrap();

        let error = credential
            .into_issued_credential(
                "key_id".to_string(),
                &holder_public_key,
                &normalized_preview,
                &[trust_anchor],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::IssuedCredentialMismatch { .. });
    }

    #[test]
    fn test_credential_response_into_mdoc_issued_doctype_mismatch_error() {
        let (credential, mut normalized_preview, holder_public_key, trust_anchor) =
            mock_credential_response_credential();

        // Converting a `CredentialResponse` into an `Mdoc` with a different doc_type in the preview than contained
        // within the response should fail.
        normalized_preview.content.credential_payload.attestation_type = String::from("other.attestation_type");

        let error = credential
            .into_issued_credential(
                "key_id".to_string(),
                &holder_public_key,
                &normalized_preview,
                &[trust_anchor],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::IssuedCredentialMismatch { .. });
    }

    #[test]
    fn test_credential_response_into_mdoc_issued_validity_info_mismatch_error() {
        let (credential, mut normalized_preview, holder_public_key, trust_anchor) =
            mock_credential_response_credential();

        // Converting a `CredentialResponse` into an `Mdoc` with different expiration information in the preview than
        // contained within the response should fail.

        normalized_preview.content.credential_payload.not_before =
            Some((Utc::now() + chrono::Duration::days(1)).into());

        let error = credential
            .into_issued_credential(
                "key_id".to_string(),
                &holder_public_key,
                &normalized_preview,
                &[trust_anchor],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::IssuedCredentialMismatch { .. });
    }

    #[test]
    fn test_credential_response_into_mdoc_issued_attestation_qualification_mismatch_error() {
        let (credential, mut normalized_preview, holder_public_key, trust_anchor) =
            mock_credential_response_credential();

        // Converting a `CredentialResponse` into an `Mdoc` with a different doc_type in the preview than contained
        // within the response should fail.
        normalized_preview.content.credential_payload.attestation_qualification = AttestationQualification::PubEAA;

        let error = credential
            .into_issued_credential(
                "key_id".to_string(),
                &holder_public_key,
                &normalized_preview,
                &[trust_anchor],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::IssuedCredentialMismatch { .. });
    }

    #[rstest]
    #[case(vec_nonempty![ClaimPath::SelectByKey("non_existing".to_string())], vec![], ExpectedResult::ObjectFieldNotFound("non_existing".parse().unwrap()))]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_value_always".to_string())], vec![vec_nonempty![ClaimPath::SelectByKey("root_value_always".to_string())]], ExpectedResult::Ok)]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_value_always".to_string())], vec![], ExpectedResult::SelectivelyDisclosability(ClaimSelectiveDisclosureMetadata::Always, false))]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_value_allow".to_string())], vec![vec_nonempty![ClaimPath::SelectByKey("root_value_allow".to_string())]], ExpectedResult::Ok)]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_value_allow".to_string())], vec![], ExpectedResult::Ok)]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_value_never".to_string())], vec![vec_nonempty![ClaimPath::SelectByKey("root_value_never".to_string())]], ExpectedResult::SelectivelyDisclosability(ClaimSelectiveDisclosureMetadata::Never, true))]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_value_never".to_string())], vec![], ExpectedResult::Ok)]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_array_always".to_string())], vec![vec_nonempty![ClaimPath::SelectByKey("root_array_always".to_string())]], ExpectedResult::Ok)]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_array_always".to_string())], vec![], ExpectedResult::SelectivelyDisclosability(ClaimSelectiveDisclosureMetadata::Always, false))]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_array_allow".to_string())], vec![vec_nonempty![ClaimPath::SelectByKey("root_array_allow".to_string())]], ExpectedResult::Ok)]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_array_allow".to_string())], vec![], ExpectedResult::Ok)]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_array_never".to_string())], vec![vec_nonempty![ClaimPath::SelectByKey("root_array_never".to_string())]], ExpectedResult::SelectivelyDisclosability(ClaimSelectiveDisclosureMetadata::Never, true))]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_array_never".to_string())], vec![], ExpectedResult::Ok)]
    fn test_verify_claim_selective_disclosability(
        #[case] claim_to_verify: VecNonEmpty<ClaimPath>,
        #[case] claims_to_conceal: Vec<VecNonEmpty<ClaimPath>>,
        #[case] expected: ExpectedResult,
    ) {
        let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_keypair = issuer_ca.generate_issuer_mock().unwrap();

        let claims_metadata: HashMap<Vec<ClaimPath>, ClaimSelectiveDisclosureMetadata> = HashMap::from_iter([
            (
                vec![ClaimPath::SelectByKey("root_value_always".to_string())],
                ClaimSelectiveDisclosureMetadata::Always,
            ),
            (
                vec![ClaimPath::SelectByKey("root_value_allow".to_string())],
                ClaimSelectiveDisclosureMetadata::Allowed,
            ),
            (
                vec![ClaimPath::SelectByKey("root_value_never".to_string())],
                ClaimSelectiveDisclosureMetadata::Never,
            ),
            (
                vec![ClaimPath::SelectByKey("root_array_always".to_string())],
                ClaimSelectiveDisclosureMetadata::Always,
            ),
            (
                vec![ClaimPath::SelectByKey("root_array_allow".to_string())],
                ClaimSelectiveDisclosureMetadata::Allowed,
            ),
            (
                vec![ClaimPath::SelectByKey("root_array_never".to_string())],
                ClaimSelectiveDisclosureMetadata::Never,
            ),
        ]);

        let signed_sd_jwt: SignedSdJwt = conceal_and_sign(
            &issuer_keypair,
            serde_json::from_value(json!({
                "vct": "com:example:1",
                "iss": "https://issuer.example.com/",
                "iat": 1683000000,
                "cnf": {
                    "jwk": {
                        "kty": "EC",
                        "crv": "P-256",
                        "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                        "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
                    }
                },
                "root_value_always": 1,
                "root_value_allow": 2,
                "root_value_never": 3,
                "root_array_always": [
                    4
                ],
                "root_array_allow": [
                    5
                ],
                "root_array_never": [
                    6
                ],
            }))
            .unwrap(),
            claims_to_conceal,
        );
        let sd_jwt: VerifiedSdJwt = signed_sd_jwt.into_verified();

        let result =
            Credential::verify_claim_selective_disclosability(&sd_jwt, claim_to_verify.as_slice(), &claims_metadata);

        match expected {
            ExpectedResult::Ok => result.unwrap(),
            ExpectedResult::ObjectFieldNotFound(expected_claim_name) => {
                let error = result.unwrap_err();
                assert_matches!(error, WalletIssuanceError::SdJwtVerification(DecoderError::ClaimStructure(
                    ClaimError::ObjectFieldNotFound(claim_name, _)
                )) if claim_name == expected_claim_name);
            }
            ExpectedResult::SelectivelyDisclosability(expected_sd, expected_disclosability) => {
                let error = result.unwrap_err();
                assert_matches!(error, WalletIssuanceError::SdJwtVerification(DecoderError::ClaimStructure(
                    ClaimError::SelectiveDisclosabilityMismatch(claim, sd, is_selective_disclosable)))
                                if claim == claim_to_verify.into_inner()
                                && expected_sd == sd
                                && expected_disclosability == is_selective_disclosable);
            }
        }
    }

    enum ExpectedResult {
        Ok,
        ObjectFieldNotFound(ClaimName),
        SelectivelyDisclosability(ClaimSelectiveDisclosureMetadata, bool),
    }
}
