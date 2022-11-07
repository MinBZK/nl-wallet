import 'package:equatable/equatable.dart';

import '../../../domain/model/data_attribute.dart';
import 'verifier.dart';
import 'verifier_policy.dart';

class VerificationRequest extends Equatable {
  final int id;
  final Verifier verifier;
  final List<DataAttribute> attributes;
  final VerifierPolicy policy;

  const VerificationRequest({
    required this.id,
    required this.verifier,
    required this.attributes,
    required this.policy,
  });

  @override
  List<Object?> get props => [id, verifier, attributes, policy];
}
