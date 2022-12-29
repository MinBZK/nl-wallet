import 'package:equatable/equatable.dart';

import '../../feature/verification/model/organization.dart';
import 'attribute/attribute.dart';
import 'attribute/data_attribute.dart';
import 'attribute/requested_attribute.dart';
import 'policy/policy.dart';
import 'wallet_card.dart';

class IssuanceFlow extends Equatable {
  final Organization organization;
  final List<Attribute> attributes;
  final Policy policy;
  final List<WalletCard> cards;

  const IssuanceFlow({
    required this.organization,
    required this.attributes,
    required this.policy,
    required this.cards,
  });

  List<DataAttribute> get resolvedAttributes => attributes.whereType<DataAttribute>().toList();

  List<RequestedAttribute> get missingAttributes => attributes.whereType<RequestedAttribute>().toList();

  @override
  List<Object?> get props => [organization, attributes, policy, cards];
}
