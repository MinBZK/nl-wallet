import 'package:equatable/equatable.dart';

import '../attribute/attribute.dart';
import '../card/wallet_card.dart';
import '../disclosure/disclosure_type.dart';
import '../document.dart';
import '../organization.dart';
import '../policy/policy.dart';

export '../disclosure/disclosure_type.dart';

part 'disclosure_event.dart';
part 'issuance_event.dart';
part 'sign_event.dart';

sealed class WalletEvent extends Equatable {
  final DateTime dateTime;
  final EventStatus status;

  List<DataAttribute> get attributes;

  const WalletEvent({
    required this.dateTime,
    required this.status,
  });

  const factory WalletEvent.disclosure({
    required DateTime dateTime,
    required EventStatus status,
    required Organization relyingParty,
    required LocalizedText purpose,
    required List<WalletCard> cards,
    required Policy policy,
    required DisclosureType type,
  }) = DisclosureEvent;

  const factory WalletEvent.issuance({
    required DateTime dateTime,
    required EventStatus status,
    required WalletCard card,
  }) = IssuanceEvent;

  const factory WalletEvent.sign({
    required DateTime dateTime,
    required EventStatus status,
    required Organization relyingParty,
    required Policy policy,
    required Document document,
  }) = SignEvent;
}

enum EventStatus { success, cancelled, error }
