import 'package:wallet_core/core.dart';

import 'mock_attributes.dart';
import 'mock_organizations.dart';

final kPidAttestations = [
  Attestation(
    identity: AttestationIdentity_Fixed(id: 'pid'),
    attestationType: kPidDocType,
    displayMetadata: [],
    issuer: kOrganizations[kRvigId]!,
    attributes: kMockPidAttestationAttributes,
  ),
  Attestation(
    identity: AttestationIdentity_Fixed(id: 'address'),
    attestationType: kAddressDocType,
    displayMetadata: [],
    issuer: kOrganizations[kRvigId]!,
    attributes: kMockAddressAttestationAttributes,
  ),
];
