import 'package:equatable/equatable.dart';

import '../../../domain/model/attribute/requested_attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';

class DisclosureRequest extends Equatable {
  final String id;
  final Organization organization;
  final List<RequestedAttribute> requestedAttributes;
  final String requestPurpose;
  final Policy interactionPolicy;

  const DisclosureRequest({
    required this.id,
    required this.organization,
    required this.requestedAttributes,
    required this.requestPurpose,
    required this.interactionPolicy,
  });

  @override
  List<Object?> get props => [
        id,
        organization,
        requestedAttributes,
        requestPurpose,
        interactionPolicy,
      ];
}
