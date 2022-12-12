import 'package:equatable/equatable.dart';

import '../../feature/verification/model/organization.dart';
import 'attribute/requested_attribute.dart';
import 'document.dart';
import 'trust_provider.dart';

class SignRequest extends Equatable {
  final String id;
  final Organization organization;
  final Document document;
  final TrustProvider trustProvider;
  final List<RequestedAttribute> requestedAttributes;
  final bool dataIsShared;

  const SignRequest({
    required this.id,
    required this.organization,
    required this.document,
    required this.trustProvider,
    required this.requestedAttributes,
    required this.dataIsShared,
  });

  @override
  List<Object?> get props => [id, organization, document, requestedAttributes, trustProvider, dataIsShared];
}
