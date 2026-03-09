use assert_matches::assert_matches;

use crate::Iso18013_5Update;

use super::Iso18013_5SessionManager;

pub async fn test_start_qr_handover<I: Iso18013_5SessionManager>() {
    let (qr, mut receiver) = I::start_qr_handover().await.unwrap();
    assert_eq!(qr, "some_qr_code");

    let update = receiver.recv().await.expect("channel closed before first update");
    assert_eq!(update, Iso18013_5Update::Connecting);

    let update = receiver.recv().await.expect("channel closed before second update");
    assert_eq!(update, Iso18013_5Update::Connected);

    let update = receiver.recv().await.expect("channel closed before third update");
    assert_matches!(
        update,
        Iso18013_5Update::DeviceRequest { session_transcript, device_request }
            if session_transcript == vec![0x01, 0x02, 0x03] && device_request == vec![0x04, 0x05, 0x06]
    );

    let update = receiver.recv().await.expect("channel closed before fourth update");
    assert_eq!(update, Iso18013_5Update::Closed);

    assert!(receiver.recv().await.is_none());
}
