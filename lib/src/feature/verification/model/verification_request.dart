import 'package:equatable/equatable.dart';

import '../../../domain/model/requested_attribute.dart';
import 'organization.dart';
import 'verifier_policy.dart';

class VerificationRequest extends Equatable {
  final String id;
  final Organization organization;
  final List<RequestedAttribute> requestedAttributes;
  final VerifierPolicy policy;

  const VerificationRequest({
    required this.id,
    required this.organization,
    required this.requestedAttributes,
    required this.policy,
  });

  @override
  List<Object?> get props => [id, organization, requestedAttributes, policy];
}
