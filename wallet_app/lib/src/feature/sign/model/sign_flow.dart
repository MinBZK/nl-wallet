import 'package:equatable/equatable.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/attribute/requested_attribute.dart';
import '../../../domain/model/document.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/trust_provider.dart';
import '../../verification/model/organization.dart';

class SignFlow extends Equatable {
  final String id;
  final Organization organization;
  final TrustProvider trustProvider;
  final Document document;
  final List<Attribute> attributes;
  final Policy policy;

  const SignFlow({
    required this.id,
    required this.organization,
    required this.trustProvider,
    required this.document,
    required this.attributes,
    required this.policy,
  });

  List<DataAttribute> get resolvedAttributes => attributes.whereType<DataAttribute>().toList();

  List<RequestedAttribute> get missingAttributes => attributes.whereType<RequestedAttribute>().toList();

  bool get hasMissingAttributes => missingAttributes.isNotEmpty;

  @override
  List<Object?> get props => [id, organization, trustProvider, document, attributes, policy];
}
