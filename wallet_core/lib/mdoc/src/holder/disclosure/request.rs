use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexSet;
use rustls_pki_types::TrustAnchor;

use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateUsage;
use utils::generator::Generator;

use crate::device_retrieval::DeviceRequest;
use crate::device_retrieval::DocRequest;
use crate::device_retrieval::ReaderAuthenticationKeyed;
use crate::engagement::SessionTranscript;
use crate::errors::Result;
use crate::holder::HolderError;
use crate::identifiers::AttributeIdentifier;
use crate::identifiers::AttributeIdentifierHolder;
use crate::utils::cose::ClonePayload;
use crate::utils::reader_auth::ReaderRegistration;
use crate::utils::serialization;
use crate::utils::serialization::CborSeq;
use crate::utils::serialization::TaggedBytes;
use crate::utils::x509::CertificateType;
use crate::ItemsRequest;

impl DeviceRequest {
    /// Verify reader authentication, if present.
    /// Note that since each DocRequest carries its own reader authentication, the spec allows the
    /// the DocRequests to be signed by distinct readers. TODO maybe support this (PVW-2368).
    /// For now, this function requires either none of the DocRequests to be signed, or all of them
    /// by the same reader.
    pub fn verify(
        &self,
        session_transcript: &SessionTranscript,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<Option<(BorrowingCertificate, ReaderRegistration)>> {
        // If there are no doc requests or none of them have reader authentication, return `None`.
        if self.doc_requests.iter().all(|d| d.reader_auth.is_none()) {
            return Ok(None);
        }

        // Otherwise, all of the doc requests need reader authentication.
        if self.doc_requests.iter().any(|d| d.reader_auth.is_none()) {
            return Err(HolderError::ReaderAuthMissing.into());
        }

        // Verify all `DocRequest` entries and make sure the resulting certificates are all exactly equal.
        let certificate = self
            .doc_requests
            .iter()
            .try_fold(None, {
                |result_cert, doc_request| -> Result<_> {
                    // This `.unwrap()` is safe, because `.verify()` will only return `None`
                    // if `reader_auth` is absent, the presence of which we checked above.
                    let doc_request_cert = doc_request.verify(session_transcript, time, trust_anchors)?.unwrap();

                    // If there is a certificate from a previous iteration, compare our certificate to that.
                    if let Some(result_cert) = result_cert {
                        if doc_request_cert != result_cert {
                            return Err(HolderError::ReaderAuthsInconsistent.into());
                        }
                    }

                    Ok(doc_request_cert.into())
                }
            })?
            .unwrap(); // This `.unwrap()` is safe for the same reason stated above.

        // Extract `ReaderRegistration` from the one certificate.
        let reader_registration = match CertificateType::from_certificate(&certificate).map_err(HolderError::from)? {
            CertificateType::ReaderAuth(Some(reader_registration)) => *reader_registration,
            _ => return Err(HolderError::NoReaderRegistration(Box::new(certificate)).into()),
        };

        // Verify that the requested attributes are included in the reader authentication.
        reader_registration
            .verify_requested_attributes(&self.items_requests())
            .map_err(HolderError::from)?;

        Ok((certificate, reader_registration).into())
    }

    pub fn items_requests(&self) -> impl Iterator<Item = &ItemsRequest> + Clone {
        self.doc_requests.iter().map(|doc_request| &doc_request.items_request.0)
    }
}

impl<'a, T: IntoIterator<Item = &'a ItemsRequest> + Clone> AttributeIdentifierHolder for T {
    fn attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.clone()
            .into_iter()
            .flat_map(|items_request| items_request.attribute_identifiers())
            .collect()
    }
}

impl DocRequest {
    pub fn verify(
        &self,
        session_transcript: &SessionTranscript,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<Option<BorrowingCertificate>> {
        // If reader authentication is present, verify it and return the certificate.
        self.reader_auth
            .as_ref()
            .map(|reader_auth| {
                // Reconstruct the reader authentication bytes for this `DocRequest`,
                // based on the item requests and session transcript.
                let reader_auth_payload = ReaderAuthenticationKeyed::new(session_transcript, &self.items_request);
                let reader_auth_payload = TaggedBytes(CborSeq(reader_auth_payload));

                // Perform verification and return the `Certificate`.
                let cose = reader_auth.clone_with_payload(serialization::cbor_serialize(&reader_auth_payload)?);
                cose.verify_against_trust_anchors(CertificateUsage::ReaderAuth, time, trust_anchors)?;
                let cert = cose.signing_cert()?;

                Ok(cert)
            })
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crypto::server_keys::generate::Ca;
    use crypto::server_keys::KeyPair;
    use utils::generator::TimeGenerator;

    use crate::errors::Error;
    use crate::iso::device_retrieval::ReaderAuthenticationBytes;
    use crate::server_keys::generate::mock::generate_reader_mock;
    use crate::utils::cose;
    use crate::utils::cose::MdocCose;

    use super::*;

    /// Create a `DocRequest` including reader authentication,
    /// based on a `SessionTranscript` and `KeyPair`.
    pub async fn create_doc_request(
        items_request: ItemsRequest,
        session_transcript: &SessionTranscript,
        private_key: &KeyPair,
    ) -> DocRequest {
        // Generate the reader authentication signature, without payload.
        let items_request = items_request.into();
        let reader_auth_keyed = ReaderAuthenticationKeyed::new(session_transcript, &items_request);

        let cose = MdocCose::<_, ReaderAuthenticationBytes>::sign(
            &TaggedBytes(CborSeq(reader_auth_keyed)),
            cose::new_certificate_header(private_key.certificate()),
            private_key,
            false,
        )
        .await
        .unwrap();
        let reader_auth = Some(cose.0.into());

        // Create and encrypt the `DeviceRequest`.
        DocRequest {
            items_request,
            reader_auth,
        }
    }

    #[tokio::test]
    async fn test_device_request_verify() {
        // Create two certificates and private keys.
        let ca = Ca::generate_reader_mock_ca().unwrap();
        let reader_registration = ReaderRegistration::new_mock();
        let private_key1 = generate_reader_mock(&ca, reader_registration.clone().into()).unwrap();
        let private_key2 = generate_reader_mock(&ca, reader_registration.clone().into()).unwrap();

        let session_transcript = SessionTranscript::new_mock();

        // Create an empty `ItemsRequest` and generate `DeviceRequest` with two `DocRequest`s
        // from it, each signed with the same certificate.
        let items_request = ItemsRequest::new_example_empty();

        let device_request = DeviceRequest::from_doc_requests(vec![
            create_doc_request(items_request.clone(), &session_transcript, &private_key1).await,
            create_doc_request(items_request.clone(), &session_transcript, &private_key1).await,
        ]);

        // Verifying this `DeviceRequest` should succeed and return the `ReaderRegistration`.
        let trust_anchors = &[ca.to_trust_anchor()];

        let verified_reader_registration = device_request
            .verify(&session_transcript, &TimeGenerator, trust_anchors)
            .expect("Could not verify DeviceRequest");

        assert_eq!(
            verified_reader_registration,
            Some((private_key1.certificate().clone(), reader_registration))
        );

        // Verifying a `DeviceRequest` that has no reader auth at all should succeed and return `None`.
        let device_request = DeviceRequest::from_items_requests(vec![items_request.clone(), items_request.clone()]);

        let no_reader_registration = device_request
            .verify(&session_transcript, &TimeGenerator, trust_anchors)
            .expect("Could not verify DeviceRequest");

        assert!(no_reader_registration.is_none());

        // Generate `DeviceRequest` with two `DocRequest`s, each signed
        // with a different key and including a different certificate.
        let device_request = DeviceRequest::from_doc_requests(vec![
            create_doc_request(items_request.clone(), &session_transcript, &private_key1).await,
            create_doc_request(items_request, &session_transcript, &private_key2).await,
        ]);

        // Verifying this `DeviceRequest` should result in a `HolderError::ReaderAuthsInconsistent` error.
        let error = device_request
            .verify(&session_transcript, &TimeGenerator, trust_anchors)
            .expect_err("Verifying DeviceRequest should have resulted in an error");

        assert_matches!(error, Error::Holder(HolderError::ReaderAuthsInconsistent));
    }

    #[tokio::test]
    async fn test_doc_request_verify() {
        // Create a CA, certificate and private key and trust anchors.
        let ca = Ca::generate_reader_mock_ca().unwrap();
        let reader_registration = ReaderRegistration::new_mock();
        let private_key = generate_reader_mock(&ca, reader_registration.into()).unwrap();
        let trust_anchors = &[ca.to_trust_anchor()];

        // Create a basic session transcript, item request and a `DocRequest`.
        let session_transcript = SessionTranscript::new_mock();
        let items_request = ItemsRequest::new_example_empty();
        let doc_request = create_doc_request(items_request.clone(), &session_transcript, &private_key).await;

        // Verification of the `DocRequest` should succeed and return the certificate contained within it.
        let certificate = doc_request
            .verify(&session_transcript, &TimeGenerator, trust_anchors)
            .expect("Could not verify DeviceRequest");

        assert_matches!(certificate, Some(cert) if cert == private_key.into());

        let other_ca = Ca::generate_reader_mock_ca().unwrap();
        let error = doc_request
            .verify(&session_transcript, &TimeGenerator, &[other_ca.to_trust_anchor()])
            .expect_err("Verifying DeviceRequest should have resulted in an error");

        assert_matches!(error, Error::Cose(_));

        // Verifying a `DocRequest` that has no reader auth should succeed and return `None`.
        let doc_request = DocRequest {
            items_request: items_request.into(),
            reader_auth: None,
        };

        let no_certificate = doc_request
            .verify(&session_transcript, &TimeGenerator, trust_anchors)
            .expect("Could not verify DeviceRequest");

        assert!(no_certificate.is_none());
    }
}
