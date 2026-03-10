use assert_matches::assert_matches;

use crate::CloseProximityDisclosureUpdate;

use super::CloseProximityDisclosureClient;

pub async fn test_start_qr_handover<I: CloseProximityDisclosureClient>() {
    let (qr, mut receiver) = I::start_qr_handover().await.unwrap();
    assert_eq!(qr, "some_qr_code");

    let update = receiver.recv().await.expect("channel closed before first update");
    assert_eq!(update, CloseProximityDisclosureUpdate::Connecting);

    let update = receiver.recv().await.expect("channel closed before second update");
    assert_eq!(update, CloseProximityDisclosureUpdate::Connected);

    let update = receiver.recv().await.expect("channel closed before third update");
    assert_matches!(
        update,
        CloseProximityDisclosureUpdate::SessionEstablished { session_transcript, device_request }
            if session_transcript == vec![0x01, 0x02, 0x03] && device_request == vec![0x04, 0x05, 0x06]
    );

    let update = receiver.recv().await.expect("channel closed before fourth update");
    assert_eq!(update, CloseProximityDisclosureUpdate::Closed);

    assert!(receiver.recv().await.is_none());
}
