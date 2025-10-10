import 'package:flutter/material.dart';

import '../../../../../domain/model/card/status/card_status.dart';
import '../../../../../domain/model/card/wallet_card.dart';
import '../../../../extension/build_context_extension.dart';
import '../../../datetime/date_formatter.dart';
import '../card_status_metadata_formatter.dart';

class CardStatusMetadataCardDataScreenFormatter implements CardStatusMetadataFormatter {
  @override
  bool show(CardStatus status) {
    return switch (status) {
      CardStatus.validSoon => true,
      CardStatus.valid => false,
      CardStatus.expiresSoon => false,
      CardStatus.expired => true,
      CardStatus.revoked => true,
      CardStatus.corrupted => true,
      CardStatus.unknown => true,
    };
  }

  @override
  String text(BuildContext context, WalletCard card) {
    return switch (card.status) {
      CardStatus.validSoon => context.l10n.cardStatusMetadataCardDataScreenValidSoon(
        DateFormatter.formatDate(context, card.validFrom),
      ),
      CardStatus.valid => '',
      CardStatus.expiresSoon => '',
      CardStatus.expired => context.l10n.cardStatusMetadataCardDataScreenExpired,
      CardStatus.revoked => context.l10n.cardStatusMetadataCardDataScreenRevoked,
      CardStatus.corrupted => context.l10n.cardStatusMetadataCardDataScreenCorrupted,
      CardStatus.unknown => context.l10n.cardStatusMetadataCardDataScreenUnknown,
    };
  }

  @override
  Color textColor(BuildContext context, CardStatus status) {
    return switch (status) {
      CardStatus.validSoon => context.colorScheme.onSurface,
      CardStatus.valid => context.colorScheme.onSurface,
      CardStatus.expiresSoon => context.colorScheme.onSurface,
      CardStatus.expired => context.colorScheme.error,
      CardStatus.revoked => context.colorScheme.error,
      CardStatus.corrupted => context.colorScheme.error,
      CardStatus.unknown => context.colorScheme.onSurface,
    };
  }

  @override
  IconData? icon(CardStatus status) {
    return switch (status) {
      CardStatus.validSoon => Icons.feedback_outlined,
      CardStatus.valid => null,
      CardStatus.expiresSoon => null,
      CardStatus.expired => Icons.event_busy,
      CardStatus.revoked => Icons.close,
      CardStatus.corrupted => Icons.block_flipped,
      CardStatus.unknown => Icons.warning_amber,
    };
  }

  @override
  Color? iconColor(BuildContext context, CardStatus status) {
    return switch (status) {
      CardStatus.validSoon => context.colorScheme.onSurfaceVariant,
      CardStatus.valid => null,
      CardStatus.expiresSoon => null,
      CardStatus.expired => context.colorScheme.error,
      CardStatus.revoked => context.colorScheme.error,
      CardStatus.corrupted => context.colorScheme.error,
      CardStatus.unknown => context.colorScheme.onSurfaceVariant,
    };
  }

  @override
  Color? backgroundColor(BuildContext context, CardStatus status) => null;
}
