import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../card/detail/argument/card_detail_screen_argument.dart';
import '../../common/widget/card/wallet_card_item.dart';
import 'notification_banner.dart';

const _iconSize = 48.0;

class CardRevocationBanner extends StatelessWidget {
  /// The card that has been revoked
  final WalletCard card;

  const CardRevocationBanner({required this.card, super.key});

  @override
  Widget build(BuildContext context) {
    return NotificationBanner(
      leadingIcon: _buildIcon(context),
      title: _buildTitle(context),
      subtitle: context.l10n.cardRevocationBannerSubtitle,
      onTap: () {
        // Animate solely by attestation id to disable SharedElementTransition in this navigation path. //PVW-5438
        final cardDetailScreenArgument = CardDetailScreenArgument.fromId(card.attestationId!, card.title);
        Navigator.restorablePushNamed(
          context,
          WalletRoutes.cardDetailRoute,
          arguments: cardDetailScreenArgument.toJson(),
        );
      },
    );
  }

  String _buildTitle(BuildContext context) {
    final cardTitle = card.title.l10nValue(context);
    return context.l10n.cardRevocationBannerTitle(cardTitle);
  }

  Widget _buildIcon(BuildContext context) {
    return Container(
      height: _iconSize,
      width: _iconSize,
      alignment: .center,
      child: SizedBox(
        height: 32,
        width: 32,
        child: WalletCardItem.fromWalletCard(context, card, showText: false),
      ),
    );
  }
}
