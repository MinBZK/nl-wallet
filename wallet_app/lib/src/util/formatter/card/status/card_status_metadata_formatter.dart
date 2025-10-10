import 'package:flutter/material.dart';

import '../../../../domain/model/card/status/card_status.dart';
import '../../../../domain/model/card/wallet_card.dart';

const kStatusWarningColor = Color(0xFF834905);

/// Abstract class that provides formatting for [CardStatus] in different contexts.
abstract class CardStatusMetadataFormatter {
  bool show(CardStatus status);

  String text(BuildContext context, WalletCard card) => throw UnimplementedError();

  Color textColor(BuildContext context, CardStatus status);

  IconData? icon(CardStatus status);

  Color? iconColor(BuildContext context, CardStatus status);

  Color? backgroundColor(BuildContext context, CardStatus status);
}
