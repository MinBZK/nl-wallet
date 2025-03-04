import 'package:equatable/equatable.dart';

import 'card/wallet_card.dart';
import 'event/wallet_event.dart';

class WalletCardDetail extends Equatable {
  final WalletCard card;
  final IssuanceEvent? mostRecentIssuance;
  final DisclosureEvent? mostRecentSuccessfulDisclosure;

  const WalletCardDetail({
    required this.card,
    required this.mostRecentIssuance,
    required this.mostRecentSuccessfulDisclosure,
  });

  @override
  List<Object?> get props => [card, mostRecentIssuance, mostRecentSuccessfulDisclosure];
}
