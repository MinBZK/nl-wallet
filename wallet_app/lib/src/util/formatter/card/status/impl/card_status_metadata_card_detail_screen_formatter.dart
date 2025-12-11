import 'package:flutter/material.dart';

import '../../../../../domain/model/attribute/attribute.dart';
import '../../../../../domain/model/card/status/card_status.dart';
import '../../../../../domain/model/card/wallet_card.dart';
import '../../../../../domain/model/organization.dart';
import '../../../../extension/build_context_extension.dart';
import '../../../datetime/date_formatter.dart';
import '../../../datetime/duration_formatter.dart';
import '../card_status_metadata_formatter.dart';

class CardStatusMetadataCardDetailScreenFormatter implements CardStatusMetadataFormatter {
  @override
  bool show(CardStatus status) => true;

  @override
  String text(BuildContext context, WalletCard card) {
    final status = card.status;
    final organisation = _formatIssuer(context, card.issuer);
    return switch (status) {
      CardStatusValidSoon() => context.l10n.cardStatusMetadataCardDetailScreenValidSoon(
        DateFormatter.formatDate(context, status.validFrom),
      ),
      CardStatusValid() => _formatCardStatusValidText(context, status),
      CardStatusExpiresSoon() => context.l10n.cardStatusMetadataCardDetailScreenExpiresSoon(
        DateFormatter.formatDateTime(context, status.validUntil),
        organisation,
        DurationFormatter.prettyPrintTimeDifference(context.l10n, status.validUntil),
      ),
      CardStatusExpired() => context.l10n.cardStatusMetadataCardDetailScreenExpired(
        DateFormatter.formatDateTime(context, status.validUntil),
        organisation,
      ),
      CardStatusRevoked() => context.l10n.cardStatusMetadataCardDetailScreenRevoked(organisation),
      CardStatusCorrupted() => context.l10n.cardStatusMetadataCardDetailScreenCorrupted(organisation),
      CardStatusUndetermined() => context.l10n.cardStatusMetadataCardDetailScreenUndetermined,
    };
  }

  String _formatCardStatusValidText(BuildContext context, CardStatusValid status) {
    final validUntil = status.validUntil;
    if (validUntil == null) return context.l10n.cardStatusMetadataCardDetailScreenValid;

    return context.l10n.cardStatusMetadataCardDetailScreenValidUntil(
      DateFormatter.formatDate(context, validUntil),
    );
  }

  @override
  Color textColor(BuildContext context, CardStatus status) {
    return switch (status) {
      CardStatusValidSoon() => context.colorScheme.onSurface,
      CardStatusValid() => context.colorScheme.onSurface,
      CardStatusExpiresSoon() => kStatusWarningColor,
      CardStatusExpired() => context.colorScheme.error,
      CardStatusRevoked() => context.colorScheme.error,
      CardStatusCorrupted() => context.colorScheme.error,
      CardStatusUndetermined() => kStatusWarningColor,
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
  Color? iconColor(BuildContext context, CardStatus status) {
    return switch (status) {
      CardStatusValidSoon() => context.colorScheme.onSurfaceVariant,
      CardStatusValid() => null,
      CardStatusExpiresSoon() => kStatusWarningColor,
      CardStatusExpired() => context.colorScheme.error,
      CardStatusRevoked() => context.colorScheme.error,
      CardStatusCorrupted() => context.colorScheme.error,
      CardStatusUndetermined() => kStatusWarningColor,
    };
  }

  @override
  Color? backgroundColor(BuildContext context, CardStatus status) => null;

  String _formatIssuer(BuildContext context, Organization issuer) => issuer.displayName.l10nValue(context);
}
