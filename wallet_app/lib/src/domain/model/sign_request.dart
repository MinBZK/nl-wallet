import 'package:equatable/equatable.dart';

import '../usecase/card/log_card_interaction_usecase.dart';
import 'attribute/missing_attribute.dart';
import 'document.dart';
import 'policy/policy.dart';
import 'trust_provider.dart';

class SignRequest extends Equatable {
  final String id;
  final Organization organization;
  final TrustProvider trustProvider;
  final Document document;
  final List<MockRequestedAttribute> requestedAttributes;
  final Policy policy;

  const SignRequest({
    required this.id,
    required this.organization,
    required this.document,
    required this.trustProvider,
    required this.requestedAttributes,
    required this.policy,
  });

  @override
  List<Object?> get props => [id, organization, document, requestedAttributes, trustProvider, policy];
}
