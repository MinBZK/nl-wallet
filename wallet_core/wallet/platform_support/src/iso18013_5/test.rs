use crate::Iso18013_5Update;

use super::Iso18013_5SessionManager;

pub async fn test_start_qr_handover<I: Iso18013_5SessionManager>() {
    let (_qr, mut receiver) = I::start_qr_handover().await.unwrap();
    let update = receiver.recv().await.expect("channel closed before first update");
    assert_eq!(update, Iso18013_5Update::Connecting);

    let update = receiver.recv().await.expect("channel closed before second update");
    assert_eq!(update, Iso18013_5Update::Connected);

    let update = receiver.recv().await.expect("channel closed before third update");
    assert!(matches!(update, Iso18013_5Update::DeviceRequest { .. }));

    let update = receiver.recv().await.expect("channel closed before fourth update");
    assert_eq!(update, Iso18013_5Update::Closed);

    assert!(receiver.recv().await.is_none());
}
