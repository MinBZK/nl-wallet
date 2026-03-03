use super::Iso18013_5SessionManager;

pub async fn test_start_qr_handover<I: Iso18013_5SessionManager>() {
    let _qr = I::start_qr_handover().await.unwrap();
}
