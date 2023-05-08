import 'package:equatable/equatable.dart';

import '../../../domain/model/attribute/requested_attribute.dart';
import '../../../domain/model/policy/policy.dart';
import 'organization.dart';

class VerificationRequest extends Equatable {
  final String id;
  final Organization organization;
  final List<RequestedAttribute> requestedAttributes;
  final Policy interactionPolicy;
  final String purpose;

  const VerificationRequest({
    required this.id,
    required this.organization,
    required this.requestedAttributes,
    required this.interactionPolicy,
    required this.purpose,
  });

  @override
  List<Object?> get props => [
        id,
        organization,
        requestedAttributes,
        interactionPolicy,
        purpose,
      ];
}
