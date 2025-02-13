import 'package:wallet_core/core.dart';

abstract class CoreMockData {
  static const Attestation attestation = Attestation(
    identity: AttestationIdentity_Ephemeral(),
    attestationType: kPidDocType,
    displayMetadata: [displayMetadata],
    issuer: organization,
    attributes: [attestationAttributeName],
  );

  static const displayMetadata =
      DisplayMetadata(lang: 'en', name: 'PID attestation', rendering: RenderingMetadata_Simple());

  static const AttestationAttribute attestationAttributeName =
      AttestationAttribute(key: 'name', labels: [], value: AttributeValue_String(value: 'Willeke'));

  static const AttestationAttribute attestationAttributeCity =
      AttestationAttribute(key: 'city', labels: [], value: AttributeValue_String(value: 'Den Haag'));

  static const Organization organization = Organization(
    legalName: [LocalizedString(language: 'en', value: 'legalName')],
    displayName: [LocalizedString(language: 'en', value: 'displayName')],
    category: [LocalizedString(language: 'en', value: 'category')],
    description: [LocalizedString(language: 'en', value: 'description')],
  );
}
