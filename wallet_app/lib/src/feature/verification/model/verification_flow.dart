import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';

import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/attribute/requested_attribute.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/wallet_card.dart';
import 'organization.dart';

class VerificationFlow extends Equatable {
  final String id;
  final Organization organization;
  final bool hasPreviouslyInteractedWithOrganization;
  final Map<WalletCard, List<DataAttribute>> availableAttributes;
  final List<RequestedAttribute> requestedAttributes;
  final String requestPurpose;
  final Policy policy;

  const VerificationFlow({
    required this.id,
    required this.organization,
    required this.hasPreviouslyInteractedWithOrganization,
    required this.availableAttributes,
    required this.requestedAttributes,
    required this.requestPurpose,
    required this.policy,
  });

  List<DataAttribute> get resolvedAttributes => availableAttributes.values.flattened.toList();

  List<RequestedAttribute> get missingAttributes => requestedAttributes
      .whereNot((requestedAttrib) => resolvedAttributes.map((attr) => attr.key).contains(requestedAttrib.key))
      .toList();

  bool get hasMissingAttributes => missingAttributes.isNotEmpty;

  @override
  List<Object?> get props => [
        id,
        organization,
        hasPreviouslyInteractedWithOrganization,
        availableAttributes,
        requestedAttributes,
        requestPurpose,
        policy,
      ];
}
