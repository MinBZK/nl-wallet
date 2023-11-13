import 'package:equatable/equatable.dart';

import 'attribute/attribute.dart';
import 'attribute/data_attribute.dart';
import 'attribute/missing_attribute.dart';
import 'organization.dart';
import 'policy/policy.dart';
import 'wallet_card.dart';

class IssuanceFlow extends Equatable {
  final Organization organization;
  final List<Attribute> attributes;
  final LocalizedText requestPurpose;
  final Policy policy;
  final List<WalletCard> cards;

  const IssuanceFlow({
    required this.organization,
    required this.attributes,
    required this.requestPurpose,
    required this.policy,
    required this.cards,
  });

  List<DataAttribute> get resolvedAttributes => attributes.whereType<DataAttribute>().toList();

  List<MissingAttribute> get missingAttributes => attributes.whereType<MissingAttribute>().toList();

  @override
  List<Object?> get props => [organization, attributes, requestPurpose, policy, cards];
}
