use std::env;
use std::fs;

use anyhow::Context;
use anyhow::Result;
use anyhow::ensure;
use wallet::hardware_test::ResponseArtifact;
use wallet::hardware_test::Verification;

fn main() -> Result<()> {
    let arguments = env::args_os().skip(1).collect::<Vec<_>>();
    ensure!(
        arguments.len() == 2,
        "usage: verify_close_proximity_response <expected.json> <received.json>"
    );
    let expected: Verification = serde_json::from_slice(&fs::read(&arguments[0])?).context("read expected session")?;
    let received: ResponseArtifact =
        serde_json::from_slice(&fs::read(&arguments[1])?).context("read reader response")?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(wallet::hardware_test::verify_response(&expected, &received))?;
    println!("Verified BLE DeviceResponse: expected disclosure or registration-certificate rejection");
    Ok(())
}
