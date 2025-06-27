import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../util/extension/object_extension.dart';
import 'card/wallet_card_item.dart';
import 'menu_item.dart';

/// A widget representing a selectable card row in the UI, displayed as a [MenuItem].
///
/// This widget is typically used to display a [WalletCard] with a title, summary, and visual
/// representation, and triggers an [onPressed] callback when interacted with.
class SelectCardRow extends StatelessWidget {
  /// The wallet card displayed in this row.
  final WalletCard card;

  /// The callback invoked when the menu item is pressed.
  final VoidCallback onPressed;

  const SelectCardRow({
    required this.card,
    required this.onPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return MenuItem(
      label: Text.rich(card.title.l10nSpan(context)),
      subtitle: Text.rich(card.summary.l10nSpan(context)).takeIf((_) => card.summary.l10nValue(context).isNotEmpty),
      leftIcon: WalletCardItem.fromWalletCard(context, card, showText: false),
      largeIcon: true,
      onPressed: onPressed,
    );
  }
}
