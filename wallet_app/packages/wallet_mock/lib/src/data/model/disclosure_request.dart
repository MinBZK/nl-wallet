import 'package:wallet_core/core.dart';

import 'requested_attribute.dart';

class DisclosureRequest {
  final String id;
  final Organization relyingParty;
  final List<RequestedAttribute> requestedAttributes;
  final String purpose;
  final RequestPolicy policy;

  DisclosureRequest({
    required this.id,
    required this.relyingParty,
    required this.requestedAttributes,
    required this.purpose,
    required this.policy,
  });
}
