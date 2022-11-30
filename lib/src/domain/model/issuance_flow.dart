import 'package:equatable/equatable.dart';

import '../../feature/verification/model/organization.dart';
import 'data_attribute.dart';
import 'wallet_card.dart';

class IssuanceFlow extends Equatable {
  final Organization organization;
  final List<DataAttribute> requestedDataAttributes;
  final List<WalletCard> cards;

  const IssuanceFlow({
    required this.organization,
    required this.requestedDataAttributes,
    required this.cards,
  });

  @override
  List<Object?> get props => [organization, requestedDataAttributes, cards];
}
