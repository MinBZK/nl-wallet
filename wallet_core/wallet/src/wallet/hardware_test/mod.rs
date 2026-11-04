//! Test-only wallet disclosure scenarios shared by the iOS XCTest harness and host verification.
//! The transport is real on the device; registration, credentials and provider services are fixtures.

mod fixture;
mod registration_certificate;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use anyhow::ensure;
use attestation_data::registration_certificate::REQUEST_INFO_WRPRC_KEY;
use attestation_types::pid_constants::PID_GIVEN_NAME;
use attestation_types::pid_constants::PID_MDOC_DOCTYPE;
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use ciborium::value::Value;
use crypto::trust_anchor::TrustAnchors;
use indexmap::IndexMap;
use mdoc::DeviceRequest;
use mdoc::DeviceResponse;
use mdoc::DeviceResponseStatus;
use mdoc::ItemsRequest;
use mdoc::SessionTranscript;
use mdoc::holder::disclosure::create_doc_request;
use mdoc::utils::serialization::cbor_deserialize;
use mdoc::utils::serialization::cbor_serialize;
use mockall::predicate::eq;
use p256::ecdsa::SigningKey;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureClient;
use serde::Deserialize;
use serde::Serialize;
use token_status_list::status_list_jwt::StatusListJwt;
use token_status_list::verification::client::mock::MockStatusListClient;
use token_status_list::verification::verifier::RevocationStatus;
use token_status_list::verification::verifier::RevocationVerifier;
use tokio::sync::mpsc;
use tokio::time::timeout;
use utils::generator::TimeGenerator;

use self::fixture::Fixture;
pub use self::registration_certificate::RegistrationCertificateCase;
use crate::DisclosureAttestationOptions;
use crate::DisclosureStatus;
use crate::WalletEvent;
use crate::storage::Storage;
use crate::wallet::CloseProximityDisclosureUpdate;
use crate::wallet::Session;

const EXPECTED_GIVEN_NAME: &str = "Bluetooth Test";
const CREDENTIAL_STATUS_LIST_URI: &str = "https://example.com/credential-status/1";
const SCENARIO_TIMEOUT: Duration = Duration::from_secs(90);

#[derive(Clone, Copy)]
enum Scenario {
    CwtDisclosure,
    JwtDisclosure,
    Rejection(RegistrationCertificateCase),
}

impl Scenario {
    fn rejection(self) -> Option<RegistrationCertificateCase> {
        match self {
            Self::Rejection(case) => Some(case),
            _ => None,
        }
    }
}

/// Public test material for the host. No private keys or real personal data are sent to the host.
#[derive(Clone, Serialize, Deserialize)]
pub struct Verification {
    issuer_trust_anchors: TrustAnchors,
    session_transcript_hex: String,
    credential_status_list: StatusListJwt,
    #[serde(default)]
    expected_error_status: Option<DeviceResponseStatus>,
}

#[derive(Serialize)]
pub struct ReaderReady {
    pub qr: String,
    pub device_request_hex: String,
    pub wallet_verification: Verification,
}

/// The Mac reader writes this only after receiving and decrypting the complete BLE response.
#[derive(Serialize, Deserialize)]
pub struct ResponseArtifact {
    pub device_response_hex: String,
    pub session_transcript_hex: String,
}

/// Exercise the public wallet disclosure API, including the signing instruction and history storage.
/// Keep the runtime alive for this entire future; the native bridge delivers updates asynchronously.
pub async fn run_disclosure<C: CloseProximityDisclosureClient>(reader_ready: impl FnOnce(ReaderReady)) -> Result<()> {
    run_scenario::<C>(Scenario::CwtDisclosure, reader_ready).await
}

/// Exercise the same disclosure with a compact JWT in the requestInfo.euWrprc byte string.
pub async fn run_disclosure_with_jwt<C: CloseProximityDisclosureClient>(
    reader_ready: impl FnOnce(ReaderReady),
) -> Result<()> {
    run_scenario::<C>(Scenario::JwtDisclosure, reader_ready).await
}

/// Run a signed reader request that must be rejected before approval or holder signing.
pub async fn run_registration_certificate_rejection<C: CloseProximityDisclosureClient>(
    case: RegistrationCertificateCase,
    reader_ready: impl FnOnce(ReaderReady),
) -> Result<()> {
    run_scenario::<C>(Scenario::Rejection(case), reader_ready)
        .await
        .with_context(|| format!("registration certificate case: {case}"))
}

async fn run_scenario<C: CloseProximityDisclosureClient>(
    scenario: Scenario,
    reader_ready: impl FnOnce(ReaderReady),
) -> Result<()> {
    let mut fixture = Fixture::<C>::new(scenario).await.context("prepare wallet fixture")?;
    let result = timeout(
        SCENARIO_TIMEOUT,
        disclose(&mut fixture, scenario.rejection(), reader_ready),
    )
    .await
    .context("wallet disclosure timed out")
    .and_then(std::convert::identity);

    // Also clean up on validation failure or timeout so the next test can advertise again.
    if let Some(Session::CloseProximityDisclosure(session)) = fixture.wallet.session.take() {
        let _ = timeout(
            Duration::from_secs(5),
            fixture.wallet.terminate_close_proximity_disclosure_session(session),
        )
        .await;
    }
    let cleanup = timeout(Duration::from_secs(5), C::stop_ble_server())
        .await
        .context("BLE cleanup timed out")
        .and_then(|result| result.context("stop BLE server"));
    result.and(cleanup)
}

async fn disclose<C: CloseProximityDisclosureClient>(
    fixture: &mut Fixture<C>,
    rejection: Option<RegistrationCertificateCase>,
    reader_ready: impl FnOnce(ReaderReady),
) -> Result<()> {
    let (sender, mut updates) = mpsc::unbounded_channel();
    let uri = fixture
        .wallet
        .start_close_proximity_disclosure(
            true,
            Box::new(move |update| {
                let _ = sender.send(update);
                Box::pin(std::future::ready(()))
            }),
        )
        .await
        .context("start disclosure")?;
    let qr = uri
        .as_ref()
        .strip_prefix("mdoc:")
        .context("missing mdoc URI scheme")?
        .to_owned();
    let session_transcript_bytes = reader_session_transcript(&qr)?;
    let session_transcript = SessionTranscript::try_from_bytes(&session_transcript_bytes)?;
    let items_request = ItemsRequest {
        doc_type: PID_MDOC_DOCTYPE.to_owned(),
        name_spaces: IndexMap::from([(
            PID_MDOC_DOCTYPE.to_owned(),
            IndexMap::from([(PID_GIVEN_NAME.to_owned(), false)]).try_into()?,
        )])
        .try_into()?,
        request_info: fixture
            .registration_certificate
            .clone()
            .map(|certificate| IndexMap::from([(REQUEST_INFO_WRPRC_KEY.to_owned(), certificate)])),
    };
    let items_requests = match rejection {
        Some(case) => case.items_requests(items_request)?,
        None => vec![items_request],
    };
    let mut doc_requests = Vec::with_capacity(items_requests.len());
    for items_request in items_requests {
        // Sign after altering the WRPRC/request, so reader authentication remains valid.
        doc_requests.push(create_doc_request(items_request, &session_transcript, &fixture.reader).await);
    }
    let request = DeviceRequest::from_doc_requests(doc_requests.try_into()?);
    fixture.verification.session_transcript_hex = hex::encode(session_transcript_bytes);
    reader_ready(ReaderReady {
        qr,
        device_request_hex: hex::encode(cbor_serialize(&request)?),
        wallet_verification: fixture.verification.clone(),
    });

    ensure!(
        matches!(updates.recv().await, Some(CloseProximityDisclosureUpdate::Connected)),
        "expected a BLE connection"
    );
    match updates.recv().await {
        Some(CloseProximityDisclosureUpdate::DeviceRequestReceived) => {}
        update => bail!("expected the reader request, received {update:?}"),
    }
    let proposal = fixture.wallet.continue_close_proximity_disclosure().await;
    if let Some(case) = rejection {
        let error = proposal.expect_err("invalid registration certificate must not produce a proposal");
        case.check_error(&error)?;
        ensure!(
            fixture.wallet.session.is_none(),
            "rejected wallet session was not cleared"
        );
        ensure!(
            !fixture
                .wallet
                .get_history(None)
                .await?
                .iter()
                .any(|event| matches!(event, WalletEvent::Disclosure { .. })),
            "rejected request must not create a disclosure history entry"
        );
        ensure!(
            !fixture
                .wallet
                .storage
                .read()
                .await
                .did_share_data_with_relying_party(&fixture.organization)
                .await?,
            "rejected request must not mark data as shared"
        );
        // Rejection aborts the wallet listener before sending the error response. Unlike success,
        // it closes the callback channel instead of publishing a Disconnected update.
        ensure!(
            updates.recv().await.is_none(),
            "rejected wallet listener was not stopped"
        );
        return Ok(());
    }
    let proposal = proposal.context("validate reader request")?;
    ensure!(
        proposal.organization == fixture.organization,
        "unexpected reader organization"
    );
    ensure!(
        !proposal.shared_data_with_relying_party_before,
        "unexpected prior disclosure"
    );
    let [DisclosureAttestationOptions::Single(attestation)] = proposal.attestation_options.as_slice() else {
        bail!("expected exactly one credential proposal");
    };
    ensure!(
        attestation.attestation_type == PID_MDOC_DOCTYPE,
        "unexpected credential type"
    );
    ensure!(
        attestation.attributes.len() == 1,
        "proposal must contain only the requested attribute"
    );
    ensure!(
        fixture
            .wallet
            .accept_disclosure(&[0], "051097".into())
            .await
            .context("accept disclosure")?
            .is_none(),
        "proximity disclosure must not return a redirect URI"
    );
    ensure!(fixture.wallet.session.is_none(), "wallet session was not cleared");
    let history = fixture.wallet.get_history(None).await?;
    let disclosures = history
        .iter()
        .filter(|event| matches!(event, WalletEvent::Disclosure { .. }))
        .collect::<Vec<_>>();
    ensure!(
        matches!(disclosures.as_slice(), [WalletEvent::Disclosure { organization, status: DisclosureStatus::Success, attestations, .. }]
            if **organization == fixture.organization && attestations.len() == 1),
        "expected one successful disclosure in history"
    );
    ensure!(
        fixture
            .wallet
            .storage
            .read()
            .await
            .did_share_data_with_relying_party(&fixture.organization)
            .await?,
        "successful disclosure was not persisted"
    );
    ensure!(
        matches!(updates.recv().await, Some(CloseProximityDisclosureUpdate::Disconnected)),
        "expected the BLE session to close"
    );
    Ok(())
}

fn reader_session_transcript(qr: &str) -> Result<Vec<u8>> {
    // Must match the public deterministic test-reader key and COSE encoding in disclosure_mac_reader.swift.
    let reader_key = SigningKey::from_slice(&hex::decode(
        "de3b4b9e5f72dd9b58406ae3091434da48a6f9fd010d88fcb0958e2cebec947c",
    )?)?;
    let point = reader_key.verifying_key().to_sec1_point(false);
    let bytes = point.as_bytes();
    let cose_key = cbor_serialize(&Value::Map(vec![
        (Value::from(1), Value::from(2)),
        (Value::from(-1), Value::from(1)),
        (Value::from(-2), Value::Bytes(bytes[1..33].to_vec())),
        (Value::from(-3), Value::Bytes(bytes[33..65].to_vec())),
    ]))?;
    Ok(cbor_serialize(&Value::Array(vec![
        Value::Tag(24, Box::new(Value::Bytes(BASE64_URL_SAFE_NO_PAD.decode(qr)?))),
        Value::Tag(24, Box::new(Value::Bytes(cose_key))),
        Value::Null,
    ]))?)
}

/// Verify the response captured by the reader, including session binding and selective disclosure.
pub async fn verify_response(expected: &Verification, received: &ResponseArtifact) -> Result<()> {
    ensure!(
        hex::decode(&received.session_transcript_hex)? == hex::decode(&expected.session_transcript_hex)?,
        "reader and wallet session transcripts differ"
    );
    let response: DeviceResponse = cbor_deserialize(hex::decode(&received.device_response_hex)?.as_slice())?;
    if let Some(status) = expected.expected_error_status {
        ensure!(
            response.status == status,
            "unexpected rejection status: {:?}",
            response.status
        );
        ensure!(
            response.documents.is_none() && response.document_errors.is_none(),
            "rejected request must return only an error status, without documents"
        );
        return Ok(());
    }
    let transcript = SessionTranscript::try_from_bytes(&hex::decode(&received.session_transcript_hex)?)?;
    let mut status_list_client = MockStatusListClient::new();
    let status_list = expected.credential_status_list.clone();
    status_list_client
        .expect_fetch()
        .with(eq(CREDENTIAL_STATUS_LIST_URI.parse::<url::Url>()?))
        .returning(move |_| Ok(status_list.clone()));
    let revocation = RevocationVerifier::new_with_defaults(Arc::new(status_list_client), TimeGenerator);
    let documents = response
        .verify(
            None,
            &transcript,
            &TimeGenerator,
            &expected.issuer_trust_anchors,
            &revocation,
        )
        .await?;
    let [document] = documents.as_slice() else {
        bail!("expected exactly one disclosed document");
    };
    ensure!(
        document.doc_type == PID_MDOC_DOCTYPE,
        "unexpected disclosed document type"
    );
    ensure!(
        document.revocation_status == Some(RevocationStatus::Valid),
        "credential status is not valid"
    );
    ensure!(
        document.attributes
            == IndexMap::from([(
                PID_MDOC_DOCTYPE.to_owned(),
                IndexMap::from([(PID_GIVEN_NAME.to_owned(), Value::Text(EXPECTED_GIVEN_NAME.to_owned()))]),
            )]),
        "response must contain exactly the requested attribute and value"
    );
    Ok(())
}
