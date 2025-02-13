import 'package:wallet_core/core.dart';

import 'mock_attributes.dart';
import 'mock_organizations.dart';

final kPidAttestations = [
  Attestation(
    identity: AttestationIdentity_Fixed(id: 'pid'),
    attestationType: kPidDocType,
    displayMetadata: [kPidDisplayMetadata],
    issuer: kOrganizations[kRvigId]!,
    attributes: kMockPidAttestationAttributes,
  ),
  Attestation(
    identity: AttestationIdentity_Fixed(id: 'address'),
    attestationType: kAddressDocType,
    displayMetadata: [kAddressDisplayMetadata],
    issuer: kOrganizations[kRvigId]!,
    attributes: kMockAddressAttestationAttributes,
  ),
];

final kPidDisplayMetadata = DisplayMetadata(lang: 'en', name: 'PID attestation', rendering: RenderingMetadata_Simple());
final kAddressDisplayMetadata =
    DisplayMetadata(lang: 'en', name: 'Address attestation', rendering: RenderingMetadata_Simple());
