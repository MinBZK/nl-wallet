use std::sync::Arc;

use tests_integration::common::*;
use wallet::AttributeValue;
use wallet::Document;

#[tokio::test]
async fn test_pid_ok() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // retain [`MockDigidSession::Context`]
    let _context = setup_digid_context();

    let pin = "112233".to_string();
    let mut wallet = setup_wallet_and_default_env(WalletDeviceVendor::Apple).await;
    wallet = do_wallet_registration(wallet, pin.clone()).await;
    wallet = do_pid_issuance(wallet, pin).await;

    // Emit documents into this local variable
    let documents: Arc<std::sync::Mutex<Vec<Document>>> = Arc::new(std::sync::Mutex::new(vec![]));
    {
        let documents = documents.clone();
        wallet
            .set_documents_callback(Box::new(move |mut d| {
                let mut documents = documents.lock().unwrap();
                documents.append(&mut d);
            }))
            .await
            .unwrap();
    }

    // Verify that the first mdoc contains the bsn
    let documents = documents.lock().unwrap();
    let pid_mdoc = documents.first().unwrap();
    let bsn_attr = pid_mdoc.attributes.iter().find(|a| *a.0 == "bsn");

    match bsn_attr {
        Some(bsn_attr) => assert_eq!(bsn_attr.1.value, AttributeValue::String("999991772".to_string())),
        None => panic!("BSN attribute not found"),
    }

    Ok(())
}
