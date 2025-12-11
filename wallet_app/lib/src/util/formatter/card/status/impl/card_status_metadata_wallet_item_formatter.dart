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
      CardStatusValidSoon() => true,
      CardStatusValid() => false,
      CardStatusExpiresSoon() => true,
      CardStatusExpired() => true,
      CardStatusRevoked() => true,
      CardStatusCorrupted() => true,
      CardStatusUndetermined() => true,
    };
  }

  @override
  String text(BuildContext context, WalletCard card) {
    final status = card.status;
    return switch (status) {
      CardStatusValidSoon() => context.l10n.cardStatusMetadataWalletItemValidSoon,
      CardStatusValid() => '',
      CardStatusExpiresSoon() => context.l10n.cardStatusMetadataWalletItemExpiresSoon(
        DurationFormatter.prettyPrintTimeDifference(context.l10n, status.validUntil),
      ),
      CardStatusExpired() => context.l10n.cardStatusMetadataWalletItemExpired,
      CardStatusRevoked() => context.l10n.cardStatusMetadataWalletItemRevoked,
      CardStatusCorrupted() => context.l10n.cardStatusMetadataWalletItemCorrupted,
      CardStatusUndetermined() => context.l10n.cardStatusMetadataWalletItemUndetermined,
    };
  }

  @override
  Color textColor(BuildContext context, CardStatus status) {
    const colorScheme = LightWalletTheme.colorScheme;
    return switch (status) {
      CardStatusValidSoon() => colorScheme.onSurface,
      CardStatusValid() => colorScheme.onSurface,
      CardStatusExpiresSoon() => colorScheme.onSurface,
      CardStatusExpired() => colorScheme.onError,
      CardStatusRevoked() => colorScheme.onError,
      CardStatusCorrupted() => colorScheme.onError,
      CardStatusUndetermined() => colorScheme.onError,
    };
  }

  @override
  IconData? icon(CardStatus status) {
    return switch (status) {
      CardStatusValidSoon() => Icons.block_flipped,
      CardStatusValid() => null,
      CardStatusExpiresSoon() => Icons.schedule,
      CardStatusExpired() => Icons.event_busy,
      CardStatusRevoked() => Icons.close,
      CardStatusCorrupted() => Icons.block_flipped,
      CardStatusUndetermined() => Icons.warning_amber,
    };
  }

  @override
  Color iconColor(BuildContext context, CardStatus status) {
    const colorScheme = LightWalletTheme.colorScheme;
    return switch (status) {
      CardStatusValidSoon() => colorScheme.onSurfaceVariant,
      CardStatusValid() => colorScheme.onSurfaceVariant,
      CardStatusExpiresSoon() => colorScheme.onSurfaceVariant,
      CardStatusExpired() => colorScheme.surface,
      CardStatusRevoked() => colorScheme.surface,
      CardStatusCorrupted() => colorScheme.surface,
      CardStatusUndetermined() => colorScheme.surface,
    };
  }

  @override
  Color? backgroundColor(BuildContext context, CardStatus status) {
    const colorScheme = LightWalletTheme.colorScheme;
    return switch (status) {
      CardStatusValidSoon() => colorScheme.surface,
      CardStatusValid() => colorScheme.surface,
      CardStatusExpiresSoon() => colorScheme.surface,
      CardStatusExpired() => colorScheme.error,
      CardStatusRevoked() => colorScheme.error,
      CardStatusCorrupted() => colorScheme.error,
      CardStatusUndetermined() => kStatusWarningColor,
    };
  }
}
