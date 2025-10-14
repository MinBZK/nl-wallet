import 'package:flutter/material.dart';

import '../../../../../domain/model/card/status/card_status.dart';
import '../../../../../domain/model/card/wallet_card.dart';
import '../../../../extension/build_context_extension.dart';
import '../card_status_metadata_formatter.dart';

class CardStatusMetadataSharedAttributesCardFormatter implements CardStatusMetadataFormatter {
  @override
  bool show(CardStatus status) {
    return switch (status) {
      CardStatus.validSoon => false,
      CardStatus.valid => false,
      CardStatus.expiresSoon => false,
      CardStatus.expired => true,
      CardStatus.revoked => true,
      CardStatus.corrupted => true,
      CardStatus.unknown => false,
    };
  }

  @override
  String text(BuildContext context, WalletCard card) {
    return switch (card.status) {
      CardStatus.validSoon => '',
      CardStatus.valid => '',
      CardStatus.expiresSoon => '',
      CardStatus.expired => context.l10n.cardStatusMetadataSharedAttributesCardExpired,
      CardStatus.revoked => context.l10n.cardStatusMetadataSharedAttributesCardRevoked,
      CardStatus.corrupted => context.l10n.cardStatusMetadataSharedAttributesCardCorrupted,
      CardStatus.unknown => '',
    };
  }

  @override
  Color textColor(BuildContext context, CardStatus status) => context.colorScheme.onSurface;

  @override
  IconData? icon(CardStatus status) => Icons.warning_amber;

  @override
  Color? iconColor(BuildContext context, CardStatus status) => context.colorScheme.onSurfaceVariant;

  @override
  Color? backgroundColor(BuildContext context, CardStatus status) => null;
}
