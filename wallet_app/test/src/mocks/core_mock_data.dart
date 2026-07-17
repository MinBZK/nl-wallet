import 'package:mockito/mockito.dart';
import 'package:wallet_core/core.dart';
import 'package:wallet_mock/mock.dart';

class MockAttestationPresentation extends Mock implements AttestationPresentation {}

abstract class CoreMockData {
  static const AttestationPresentation attestation = AttestationPresentation(
    identity: AttestationIdentity_Ephemeral(),
    attestationType: MockAttestationTypes.pid,
    format: .SdJwt,
    displayMetadata: [enDisplayMetadata, nlDisplayMetadata],
    issuer: organization,
    attributes: [attestationAttributeName],
    validityStatus: ValidityStatus_Valid(validUntil: null),
  );

  static const AttestationPresentation altAttestation = AttestationPresentation(
    identity: AttestationIdentity_Ephemeral(),
    attestationType: MockAttestationTypes.pid,
    format: .MsoMdoc,
    displayMetadata: [enDisplayMetadata, nlDisplayMetadata],
    issuer: organization,
    attributes: [attestationAttributeName, attestationAttributeCity],
    validityStatus: ValidityStatus_Valid(validUntil: '2050-06-08T19:46:03Z'),
  );

  static const enDisplayMetadata = DisplayMetadata(locale: 'en', name: 'PID attestation', rendering: null);
  static const nlDisplayMetadata = DisplayMetadata(locale: 'nl', name: 'PID attestatie', rendering: null);

  static const AttestationAttribute attestationAttributeName = AttestationAttribute(
    key: 'name',
    labels: [],
    value: AttributeValue_String(value: 'Willeke'),
  );

  static const AttestationAttribute attestationAttributeCity = AttestationAttribute(
    key: 'city',
    labels: [],
    value: AttributeValue_String(value: 'Den Haag'),
  );

  static const Organization organization = Organization(
    legalName: 'legalName',
    displayName: 'displayName',
    description: [LocalizedString(language: 'en', value: 'description')],
    identifier: 'identifier',
    category: [LocalizedString(language: 'en', value: 'category')],
    countryCode: 'NL',
  );

  static const RequestPolicy policy = RequestPolicy(
    dataSharedWithThirdParties: true,
    dataDeletionPossible: true,
    policyUrl: 'https://example.org',
  );

  static const FlutterConfiguration flutterConfiguration = FlutterConfiguration(
    inactiveWarningTimeout: 0,
    inactiveLockTimeout: 0,
    backgroundLockTimeout: 0,
    pidAttestations: [],
    staticAssetsBaseUrl: '',
    version: '0',
    environment: 'test',
  );
}
