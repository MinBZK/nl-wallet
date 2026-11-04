import 'package:flutter/material.dart';

import '../../../../domain/model/card/status/card_status.dart';
import '../../../../domain/model/card/wallet_card.dart';

const kStatusWarningColorLight = Color(0xFF834905);
const kStatusWarningColorDark = Color(0xFFFFBB6A);

/// Abstract class that provides formatting for [CardStatus] in different contexts.
abstract class CardStatusMetadataFormatter {
  bool show(CardStatus status);

  String text(BuildContext context, WalletCard card, {bool isPidCard = false});

  Color textColor(BuildContext context, CardStatus status);

  IconData? icon(CardStatus status);

  /// Names the status for assistive tech when [text] alone doesn't, so the state isn't conveyed by the icon only.
  String? iconSemanticsLabel(BuildContext context, WalletCard card);

  Color? iconColor(BuildContext context, CardStatus status);

  Color? backgroundColor(BuildContext context, CardStatus status);
}
