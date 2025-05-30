class MockConstants {
  const MockConstants._();

  // Hardcoded docTypes, these are exposed here because the card data is still enriched
  // based on this docType inside wallet_app (see [CardFrontMapper]). To be removed #someday
  static const pidDocType = 'mock.pid.id';
  static const addressDocType = 'mock.pid.address';
  static const drivingLicenseDocType = 'com.example.drivinglicense';

  static const pidIssuanceRedirectUri = 'initiate_mock_digid_flow';
  static const versionString = '0.0.0-mock';
}
