class MockConstants {
  const MockConstants._();

  static const pidIssuanceRedirectUri = 'initiate_mock_digid_flow';
  static const pidRenewalRedirectUri = 'initiate_mock_renew_digid_flow';
  static const pinRecoveryRedirectUri = 'initiate_mock_pin_recovery_flow';
  static const versionString = '0.0.0-mock';
}

class MockAttestationTypes {
  /// Some hardcoded attestation types used by the mock build of the app.
  /// Used in both [MockCardBackground] and [MockCardHolograph] to determine
  /// the visual representation of the card.
  const MockAttestationTypes._();

  static const pid = 'mock.pid.id';
  static const address = 'mock.pid.address';
  static const drivingLicense = 'com.example.driving_license';
  static const healthInsurance = 'com.example.health_insurance';
  static const certificateOfConduct = 'com.example.certificate_of_conduct';
  static const bscDiploma = 'com.example.bsc_diploma';
  static const mscDiploma = 'com.example.msc_diploma';
}
