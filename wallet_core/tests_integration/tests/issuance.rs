use std::sync::Arc;

use tests_integration::common::*;
use wallet::openid4vc::AttributeValue;
use wallet::Attestation;
use wallet::AttestationAttributeValue;

#[tokio::test]
async fn test_pid_ok() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // retain [`MockDigidSession::Context`]
    let _context = setup_digid_context();

    let pin = "112233";
    let (mut wallet, _) = setup_wallet_and_default_env(WalletDeviceVendor::Apple).await;
    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin).await;

    // Emit attestations into this local variable
    let attestations: Arc<std::sync::Mutex<Vec<Attestation>>> = Arc::new(std::sync::Mutex::new(vec![]));
    {
        let attestations = attestations.clone();
        wallet
            .set_attestations_callback(Box::new(move |mut a| {
                let mut attestations = attestations.lock().unwrap();
                attestations.append(&mut a);
            }))
            .await
            .unwrap();
    }

    // Verify that the first mdoc contains the bsn
    let attestations = attestations.lock().unwrap();
    let pid_attestation = attestations.first().unwrap();
    let bsn_attr = pid_attestation.attributes.iter().find(|a| a.key == vec!["bsn"]);

    match bsn_attr {
        Some(bsn_attr) => assert_eq!(
            bsn_attr.value,
            AttestationAttributeValue::Basic(AttributeValue::Text("999991772".to_string()))
        ),
        None => panic!("BSN attribute not found"),
    }

    Ok(())
}
