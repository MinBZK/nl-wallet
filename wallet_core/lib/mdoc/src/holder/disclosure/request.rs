use chrono::DateTime;
use chrono::Utc;
use rustls_pki_types::TrustAnchor;

use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateUsage;
use utils::generator::Generator;

use crate::device_retrieval::DeviceRequest;
use crate::device_retrieval::DocRequest;
use crate::device_retrieval::ReaderAuthenticationKeyed;
use crate::engagement::SessionTranscript;
use crate::errors::Result;
use crate::utils::cose::ClonePayload;
use crate::utils::serialization;
use crate::utils::serialization::CborSeq;
use crate::utils::serialization::TaggedBytes;
use crate::ItemsRequest;

impl DeviceRequest {
    pub fn items_requests(&self) -> impl Iterator<Item = &ItemsRequest> + Clone {
        self.doc_requests.iter().map(|doc_request| &doc_request.items_request.0)
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
    use crate::test::generate_reader_mock;
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
    async fn test_doc_request_verify() {
        // Create a CA, certificate and private key and trust anchors.
        let ca = Ca::generate_reader_mock_ca().unwrap();
        let private_key = generate_reader_mock(&ca).unwrap();
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
