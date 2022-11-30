import 'package:equatable/equatable.dart';

import '../../../domain/model/data_attribute.dart';
import 'organization.dart';
import 'verifier_policy.dart';

class VerificationFlow extends Equatable {
  final String id;
  final Organization organization;
  final List<DataAttribute> requestedDataAttributes;
  final VerifierPolicy policy;

  const VerificationFlow({
    required this.id,
    required this.organization,
    required this.requestedDataAttributes,
    required this.policy,
  });

  bool get hasMissingAttributes => requestedDataAttributes.any((element) => element.value == null);

  @override
  List<Object?> get props => [id, organization, requestedDataAttributes, policy];
}
