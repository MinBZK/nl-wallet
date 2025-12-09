import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../card/detail/argument/card_detail_screen_argument.dart';
import '../../common/widget/card/wallet_card_item.dart';
import 'notification_banner.dart';

const _iconSize = 48.0;

/// A banner displayed to notify the user about an expiring or already expired card.
///
/// This widget is a specialized [NotificationBanner] that shows card-specific
/// information. It indicates whether the card is about to expire or has already
/// expired. Tapping the banner navigates the user to the card's detail screen.
class CardExpiryBanner extends StatelessWidget {
  /// The card that is expiring or has expired.
  final WalletCard card;

  /// The remaining duration until the card expires.
  ///
  /// If the duration is zero or negative, the card is considered expired.
  /// Defaults to [Duration.zero], treating the card as expired.
  final Duration expiresIn;

  /// Returns `true` if the card has expired.
  bool get isExpired => expiresIn <= Duration.zero;

  const CardExpiryBanner({required this.card, this.expiresIn = Duration.zero, super.key});

  @override
  Widget build(BuildContext context) {
    return NotificationBanner(
      leadingIcon: _buildIcon(context),
      title: _buildTitle(context),
      subtitle: context.l10n.cardExpiryBannerSubtitle,
      onTap: () {
        Navigator.restorablePushNamed(
          context,
          WalletRoutes.cardDetailRoute,
          arguments: CardDetailScreenArgument.forCard(card).toJson(),
        );
      },
    );
  }

  /// Builds the title string based on the expiry status.
  String _buildTitle(BuildContext context) {
    final cardTitle = card.title.l10nValue(context);
    if (isExpired) return context.l10n.cardExpiryBannerExpiredTitle(cardTitle);
    return context.l10n.cardExpiryBannerExpiresSoonTitle(cardTitle, math.max(expiresIn.inDays, 0 /* sane fallback */));
  }

  /// Builds the icon for the banner, which includes a miniature version of the
  /// card and an optional warning symbol if the card has expired.
  Widget _buildIcon(BuildContext context) {
    final walletCard = SizedBox(
      height: 32,
      width: 32,
      child: WalletCardItem.fromWalletCard(context, card, showText: false),
    );
    final warningIcon = Icon(Icons.error_outlined, color: context.colorScheme.error, size: 20);
    return SizedBox(
      height: _iconSize,
      width: _iconSize,
      child: Stack(
        children: [
          Center(child: walletCard),
          Align(
            alignment: Alignment.bottomRight,
            child: isExpired ? warningIcon : null,
          ),
        ],
      ),
    );
  }
}
