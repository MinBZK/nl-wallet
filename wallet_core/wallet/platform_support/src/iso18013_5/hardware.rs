use crate::bridge::iso18013_5::get_iso18013_5_bridge;
use crate::iso18013_5::Iso18013_5Error;

use super::Iso18013_5SessionManager;

struct HardwareIso18013_5SessionManager;

impl Iso18013_5SessionManager for HardwareIso18013_5SessionManager {
    async fn start_qr_handover() -> Result<String, Iso18013_5Error> {
        get_iso18013_5_bridge().start_qr_handover().await
    }
}
