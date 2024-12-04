/// This is an in-memory implementation of the rust provided wallet_core api
library;

export 'src/wallet_core.dart';
// Mock specific exports
export 'src/wallet_core_for_issuance.dart';
export 'src/wallet_core_for_signing.dart';

const kMockPidIssuanceRedirectUri = 'initiate_mock_digid_flow';
const kDrivingLicenseDocType = 'com.example.drivinglicense';
const kMockVersionString =
    '0.2.3-dev (linux/x86_64, debug-mode, built: Tue, 3 Dec 2024 10:26:24 +0000, commit: x0y1234+modifications)';
