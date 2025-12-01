import 'package:flutter/material.dart';

import '../../../../../domain/model/card/status/card_status.dart';
import '../../../../../domain/model/card/wallet_card.dart';
import '../../../../../theme/light_wallet_theme.dart';
import '../../../../extension/build_context_extension.dart';
import '../../../datetime/duration_formatter.dart';
import '../card_status_metadata_formatter.dart';

class CardStatusMetadataWalletItemFormatter implements CardStatusMetadataFormatter {
  @override
  bool show(CardStatus status) {
    return switch (status) {
      CardStatus.validSoon => true,
      CardStatus.valid => false,
      CardStatus.expiresSoon => true,
      CardStatus.expired => true,
      CardStatus.revoked => true,
      CardStatus.corrupted => true,
      CardStatus.unknown => true,
    };
  }

  @override
  String text(BuildContext context, WalletCard card) {
    return switch (card.status) {
      CardStatus.validSoon => context.l10n.cardStatusMetadataWalletItemValidSoon,
      CardStatus.valid => '',
      CardStatus.expiresSoon => context.l10n.cardStatusMetadataWalletItemExpiresSoon(
        DurationFormatter.prettyPrintTimeDifference(context.l10n, card.validUntil),
      ),
      CardStatus.expired => context.l10n.cardStatusMetadataWalletItemExpired,
      CardStatus.revoked => context.l10n.cardStatusMetadataWalletItemRevoked,
      CardStatus.corrupted => context.l10n.cardStatusMetadataWalletItemCorrupted,
      CardStatus.unknown => context.l10n.cardStatusMetadataWalletItemUnknown,
    };
  }

  @override
  Color textColor(BuildContext context, CardStatus status) {
    const colorScheme = LightWalletTheme.colorScheme;
    return switch (status) {
      CardStatus.validSoon => colorScheme.onSurface,
      CardStatus.valid => colorScheme.onSurface,
      CardStatus.expiresSoon => colorScheme.onSurface,
      CardStatus.expired => colorScheme.onError,
      CardStatus.revoked => colorScheme.onError,
      CardStatus.corrupted => colorScheme.onError,
      CardStatus.unknown => colorScheme.onError,
    };
  }

  @override
  IconData? icon(CardStatus status) {
    return switch (status) {
      CardStatus.validSoon => Icons.block_flipped,
      CardStatus.valid => null,
      CardStatus.expiresSoon => Icons.schedule,
      CardStatus.expired => Icons.event_busy,
      CardStatus.revoked => Icons.close,
      CardStatus.corrupted => Icons.block_flipped,
      CardStatus.unknown => Icons.warning_amber,
    };
  }

  @override
  Color iconColor(BuildContext context, CardStatus status) {
    const colorScheme = LightWalletTheme.colorScheme;
    return switch (status) {
      CardStatus.validSoon => colorScheme.onSurfaceVariant,
      CardStatus.valid => colorScheme.onSurfaceVariant,
      CardStatus.expiresSoon => colorScheme.onSurfaceVariant,
      CardStatus.expired => colorScheme.surface,
      CardStatus.revoked => colorScheme.surface,
      CardStatus.corrupted => colorScheme.surface,
      CardStatus.unknown => colorScheme.surface,
    };
  }

  @override
  Color? backgroundColor(BuildContext context, CardStatus status) {
    const colorScheme = LightWalletTheme.colorScheme;
    return switch (status) {
      CardStatus.validSoon => colorScheme.surface,
      CardStatus.valid => colorScheme.surface,
      CardStatus.expiresSoon => colorScheme.surface,
      CardStatus.expired => colorScheme.error,
      CardStatus.revoked => colorScheme.error,
      CardStatus.corrupted => colorScheme.error,
      CardStatus.unknown => kStatusWarningColor,
    };
  }
}
