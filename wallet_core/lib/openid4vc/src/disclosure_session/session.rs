use std::hash::Hash;

use itertools::Itertools;
use tracing::info;
use tracing::warn;

use crypto::factory::KeyFactory;
use crypto::utils::random_string;
use crypto::CredentialEcdsaKey;
use http_utils::urls::BaseUrl;
use mdoc::holder::disclosure::attribute_paths_to_mdoc_paths;
use mdoc::holder::Mdoc;
use mdoc::iso::disclosure::DeviceResponse;
use mdoc::iso::engagement::SessionTranscript;
use poa::factory::PoaFactory;
use utils::vec_at_least::VecAtLeastTwoUnique;
use utils::vec_at_least::VecNonEmpty;

use crate::openid4vp::IsoVpAuthorizationRequest;
use crate::openid4vp::VpAuthorizationResponse;
use crate::verifier::SessionType;

use super::error::DisclosureError;
use super::error::VpClientError;
use super::error::VpSessionError;
use super::message_client::VpMessageClient;
use super::AttestationAttributePaths;
use super::DisclosureSession;
use super::VerifierCertificate;

#[derive(Debug)]
pub struct VpDisclosureSession<H> {
    client: H,
    session_type: SessionType,
    requested_attribute_paths: AttestationAttributePaths,
    verifier_certificate: VerifierCertificate,
    auth_request: IsoVpAuthorizationRequest,
}

impl<H> VpDisclosureSession<H> {
    pub(super) fn new(
        client: H,
        session_type: SessionType,
        requested_attribute_paths: AttestationAttributePaths,
        verifier_certificate: VerifierCertificate,
        auth_request: IsoVpAuthorizationRequest,
    ) -> Self {
        Self {
            client,
            session_type,
            requested_attribute_paths,
            verifier_certificate,
            auth_request,
        }
    }
}

impl<H> DisclosureSession for VpDisclosureSession<H>
where
    H: VpMessageClient,
{
    fn session_type(&self) -> SessionType {
        self.session_type
    }

    fn requested_attribute_paths(&self) -> &AttestationAttributePaths {
        &self.requested_attribute_paths
    }

    fn verifier_certificate(&self) -> &VerifierCertificate {
        &self.verifier_certificate
    }

    async fn terminate(self) -> Result<Option<BaseUrl>, VpSessionError> {
        let return_url = self.client.terminate(self.auth_request.response_uri).await?;

        Ok(return_url)
    }

    async fn disclose<K, KF>(
        self,
        mdocs: VecNonEmpty<Mdoc>,
        key_factory: &KF,
    ) -> Result<Option<BaseUrl>, (Self, DisclosureError<VpSessionError>)>
    where
        K: CredentialEcdsaKey + Eq + Hash,
        KF: KeyFactory<Key = K> + PoaFactory<Key = K>,
    {
        info!("disclose mdoc documents");

        let mdoc_nonce = random_string(32);
        let session_transcript = SessionTranscript::new_oid4vp(
            &self.auth_request.response_uri,
            &self.auth_request.client_id,
            self.auth_request.nonce.clone(),
            &mdoc_nonce,
        );

        // Remove the attributes from the modcs that were not requested
        // and filter out any empty mdocs resulting from this.
        let filtered_mdocs = mdocs
            .into_iter()
            .filter_map(|mut mdoc| {
                let paths = attribute_paths_to_mdoc_paths(&self.requested_attribute_paths, &mdoc.mso.doc_type);

                (!paths.is_empty()).then(|| {
                    mdoc.issuer_signed = mdoc.issuer_signed.into_attribute_subset(&paths);

                    mdoc
                })
            })
            .collect_vec();

        // Sign Document values based on the remaining contents of these mdocs and retain the keys used for signing.
        info!("signing disclosed mdoc documents");

        let result = DeviceResponse::sign_from_mdocs(filtered_mdocs, &session_transcript, key_factory).await;
        let (device_response, keys) = match result {
            Ok(value) => value,
            Err(error) => {
                return Err((
                    self,
                    DisclosureError::before_sharing(VpClientError::DeviceResponse(error).into()),
                ))
            }
        };

        // If at least two keys were used, generate a PoA to include in the response.
        let result = match VecAtLeastTwoUnique::try_from(keys.iter().collect_vec()) {
            Ok(keys) => {
                info!("creating Proof of Association");
                Some(
                    key_factory
                        .poa(keys, self.auth_request.client_id.clone(), Some(mdoc_nonce.clone()))
                        .await,
                )
            }
            Err(_) => None,
        }
        .transpose();
        let poa = match result {
            Ok(value) => value,
            Err(error) => {
                return Err((
                    self,
                    DisclosureError::before_sharing(VpClientError::Poa(Box::new(error)).into()),
                ))
            }
        };

        // Finally, encrypt the response and send it to the verifier.
        let result = VpAuthorizationResponse::new_encrypted(device_response, &self.auth_request, &mdoc_nonce, poa);
        let jwe = match result {
            Ok(value) => value,
            Err(error) => {
                return Err((
                    self,
                    DisclosureError::before_sharing(VpClientError::AuthResponseEncryption(error).into()),
                ))
            }
        };

        info!("sending Authorization Response to verifier");

        let result = self
            .client
            .send_authorization_response(self.auth_request.response_uri.clone(), jwe)
            .await
            .inspect_err(|err| {
                warn!("sending Authorization Response failed: {err}");
            });
        let redirect_uri = match result {
            Ok(value) => value,
            Err(error) => return Err((self, error.into())),
        };

        info!("sending Authorization Response succeeded");

        Ok(redirect_uri)
    }
}

#[cfg(test)]
mod tests {
    // TODO: Implement tests for VpDisclosureSession().
}
