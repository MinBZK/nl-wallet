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
      CardStatusValidSoon() => true,
      CardStatusValid() => false,
      CardStatusExpiresSoon() => false,
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
      CardStatusValidSoon() => context.l10n.cardStatusMetadataCardDataScreenValidSoon(
        DateFormatter.formatDate(context, status.validFrom),
      ),
      CardStatusValid() => '',
      CardStatusExpiresSoon() => '',
      CardStatusExpired() => context.l10n.cardStatusMetadataCardDataScreenExpired,
      CardStatusRevoked() => context.l10n.cardStatusMetadataCardDataScreenRevoked,
      CardStatusCorrupted() => context.l10n.cardStatusMetadataCardDataScreenCorrupted,
      CardStatusUndetermined() => context.l10n.cardStatusMetadataCardDataScreenUndetermined,
    };
  }

  @override
  Color textColor(BuildContext context, CardStatus status) {
    return switch (status) {
      CardStatusValidSoon() => context.colorScheme.onSurface,
      CardStatusValid() => context.colorScheme.onSurface,
      CardStatusExpiresSoon() => context.colorScheme.onSurface,
      CardStatusExpired() => context.colorScheme.error,
      CardStatusRevoked() => context.colorScheme.error,
      CardStatusCorrupted() => context.colorScheme.error,
      CardStatusUndetermined() => context.colorScheme.onSurface,
    };
  }

  @override
  IconData? icon(CardStatus status) {
    return switch (status) {
      CardStatusValidSoon() => Icons.feedback_outlined,
      CardStatusValid() => null,
      CardStatusExpiresSoon() => null,
      CardStatusExpired() => Icons.event_busy,
      CardStatusRevoked() => Icons.close,
      CardStatusCorrupted() => Icons.block_flipped,
      CardStatusUndetermined() => Icons.warning_amber,
    };
  }

  @override
  Color? iconColor(BuildContext context, CardStatus status) {
    return switch (status) {
      CardStatusValidSoon() => context.colorScheme.onSurfaceVariant,
      CardStatusValid() => null,
      CardStatusExpiresSoon() => null,
      CardStatusExpired() => context.colorScheme.error,
      CardStatusRevoked() => context.colorScheme.error,
      CardStatusCorrupted() => context.colorScheme.error,
      CardStatusUndetermined() => context.colorScheme.onSurfaceVariant,
    };
  }

  @override
  Color? backgroundColor(BuildContext context, CardStatus status) => null;
}
