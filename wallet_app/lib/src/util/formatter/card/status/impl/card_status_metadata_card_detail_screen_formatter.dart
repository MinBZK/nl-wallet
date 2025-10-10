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
    final organisation = _formatIssuer(context, card.issuer);
    return switch (card.status) {
      CardStatus.validSoon => context.l10n.cardStatusMetadataCardDetailScreenValidSoon(
        DateFormatter.formatDate(context, card.validFrom),
      ),
      CardStatus.valid => context.l10n.cardStatusMetadataCardDetailScreenValid(
        DateFormatter.formatDate(context, card.validUntil),
      ),
      CardStatus.expiresSoon => context.l10n.cardStatusMetadataCardDetailScreenExpiresSoon(
        DateFormatter.formatDateTime(context, card.validUntil),
        organisation,
        DurationFormatter.prettyPrintTimeDifference(context.l10n, card.validUntil),
      ),
      CardStatus.expired => context.l10n.cardStatusMetadataCardDetailScreenExpired(
        DateFormatter.formatDateTime(context, card.validUntil),
        organisation,
      ),
      CardStatus.revoked => context.l10n.cardStatusMetadataCardDetailScreenRevoked(organisation),
      CardStatus.corrupted => context.l10n.cardStatusMetadataCardDetailScreenCorrupted(organisation),
      CardStatus.unknown => context.l10n.cardStatusMetadataCardDetailScreenUnknown,
    };
  }

  @override
  Color textColor(BuildContext context, CardStatus status) {
    return switch (status) {
      CardStatus.validSoon => context.colorScheme.onSurface,
      CardStatus.valid => context.colorScheme.onSurface,
      CardStatus.expiresSoon => kStatusWarningColor,
      CardStatus.expired => context.colorScheme.error,
      CardStatus.revoked => context.colorScheme.error,
      CardStatus.corrupted => context.colorScheme.error,
      CardStatus.unknown => kStatusWarningColor,
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
  Color? iconColor(BuildContext context, CardStatus status) {
    return switch (status) {
      CardStatus.validSoon => context.colorScheme.onSurfaceVariant,
      CardStatus.valid => null,
      CardStatus.expiresSoon => kStatusWarningColor,
      CardStatus.expired => context.colorScheme.error,
      CardStatus.revoked => context.colorScheme.error,
      CardStatus.corrupted => context.colorScheme.error,
      CardStatus.unknown => kStatusWarningColor,
    };
  }

  @override
  Color? backgroundColor(BuildContext context, CardStatus status) => null;

  String _formatIssuer(BuildContext context, Organization issuer) => issuer.displayName.l10nValue(context);
}
