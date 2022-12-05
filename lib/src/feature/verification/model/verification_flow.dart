import 'package:equatable/equatable.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/attribute/requested_attribute.dart';
import 'organization.dart';
import 'verifier_policy.dart';

class VerificationFlow extends Equatable {
  final String id;
  final Organization organization;
  final List<Attribute> attributes;
  final VerifierPolicy policy;

  const VerificationFlow({
    required this.id,
    required this.organization,
    required this.attributes,
    required this.policy,
  });

  List<DataAttribute> get resolvedAttributes => attributes.whereType<DataAttribute>().toList();

  List<RequestedAttribute> get missingAttributes => attributes.whereType<RequestedAttribute>().toList();

  bool get hasMissingAttributes => missingAttributes.isNotEmpty;

  @override
  List<Object?> get props => [id, organization, attributes, policy];
}
