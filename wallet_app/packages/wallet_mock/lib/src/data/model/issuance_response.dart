import 'package:wallet_core/core.dart';

import 'requested_attribute.dart';

class IssuanceResponse {
  final String id;
  final Organization organization;
  final List<RequestedAttribute> requestedAttributes;
  final List<LocalizedString> requestPurpose;
  final RequestPolicy policy;
  final List<Attestation> attestations;

  const IssuanceResponse({
    required this.id,
    required this.organization,
    required this.requestedAttributes,
    required this.requestPurpose,
    required this.policy,
    required this.attestations,
  });
}
