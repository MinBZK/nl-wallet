import 'package:equatable/equatable.dart';

import '../../feature/verification/model/organization.dart';
import 'attribute/requested_attribute.dart';
import 'document.dart';
import 'policy/interaction_policy.dart';
import 'trust_provider.dart';

class SignRequest extends Equatable {
  final String id;
  final Organization organization;
  final TrustProvider trustProvider;
  final Document document;
  final List<RequestedAttribute> requestedAttributes;
  final InteractionPolicy interactionPolicy;

  const SignRequest({
    required this.id,
    required this.organization,
    required this.document,
    required this.trustProvider,
    required this.requestedAttributes,
    required this.interactionPolicy,
  });

  @override
  List<Object?> get props => [id, organization, document, requestedAttributes, trustProvider, interactionPolicy];
}
