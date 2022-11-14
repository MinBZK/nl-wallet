import 'package:equatable/equatable.dart';

import '../../../domain/model/data_attribute.dart';
import 'organization.dart';
import 'verifier_policy.dart';

class VerificationRequest extends Equatable {
  final String id;
  final Organization organization;
  final List<DataAttribute> attributes;
  final VerifierPolicy policy;

  const VerificationRequest({
    required this.id,
    required this.organization,
    required this.attributes,
    required this.policy,
  });

  bool get hasMissingAttributes => attributes.any((element) => element.value == null);

  @override
  List<Object?> get props => [id, organization, attributes, policy];
}
