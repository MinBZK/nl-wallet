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
  String text(BuildContext context, WalletCard card, {bool isPidCard = false}) {
    final status = card.status;
    final organisation = _formatIssuer(context, card.issuer);
    return switch (status) {
      CardStatusValidSoon() => context.l10n.cardStatusMetadataCardDetailScreenValidSoon(
        DateFormatter.formatDate(context, status.validFrom),
      ),
      CardStatusValid() => _formatCardStatusValidText(context, status),
      CardStatusExpiresSoon() => _formatCardStatusExpiresSoonText(context, status.validUntil, organisation, isPidCard),
      CardStatusExpired() => _formatCardStatusExpiredText(context, status.validUntil, organisation, isPidCard),
      CardStatusRevoked() => _formatCardStatusRevokedText(context, organisation, isPidCard),
      CardStatusCorrupted() => _formatCardStatusCorruptedText(context, organisation, isPidCard),
      CardStatusUndetermined() => _formatCardStatusUndeterminedText(context, isPidCard),
    };
  }

  String _formatCardStatusValidText(BuildContext context, CardStatusValid status) {
    final validUntil = status.validUntil;
    if (validUntil == null) return context.l10n.cardStatusMetadataCardDetailScreenValid;

    return context.l10n.cardStatusMetadataCardDetailScreenValidUntil(
      DateFormatter.formatDate(context, validUntil),
    );
  }

  String _formatCardStatusExpiresSoonText(
    BuildContext context,
    DateTime validUntil,
    String organisation,
    bool isPidCard,
  ) {
    final validUntilText = DateFormatter.formatDateTime(context, validUntil);
    final timeUntilText = DurationFormatter.prettyPrintTimeDifference(context.l10n, validUntil);
    return isPidCard
        ? context.l10n.cardStatusMetadataCardDetailScreenExpiresSoonPid(validUntilText, timeUntilText)
        : context.l10n.cardStatusMetadataCardDetailScreenExpiresSoon(validUntilText, organisation, timeUntilText);
  }

  String _formatCardStatusExpiredText(BuildContext context, DateTime validUntil, String organisation, bool isPidCard) {
    final validUntilText = DateFormatter.formatDateTime(context, validUntil);
    return isPidCard
        ? context.l10n.cardStatusMetadataCardDetailScreenExpiredPid(validUntilText)
        : context.l10n.cardStatusMetadataCardDetailScreenExpired(validUntilText, organisation);
  }

  String _formatCardStatusRevokedText(BuildContext context, String organisation, bool isPidCard) {
    return isPidCard
        ? context.l10n.cardStatusMetadataCardDetailScreenRevokedPid(organisation)
        : context.l10n.cardStatusMetadataCardDetailScreenRevoked(organisation);
  }

  String _formatCardStatusCorruptedText(BuildContext context, String organisation, bool isPidCard) {
    return isPidCard
        ? context.l10n.cardStatusMetadataCardDetailScreenCorruptedPid
        : context.l10n.cardStatusMetadataCardDetailScreenCorrupted(organisation);
  }

  String _formatCardStatusUndeterminedText(BuildContext context, bool isPidCard) {
    return isPidCard
        ? context.l10n.cardStatusMetadataCardDetailScreenUndeterminedPid
        : context.l10n.cardStatusMetadataCardDetailScreenUndetermined;
  }

  @override
  Color textColor(BuildContext context, CardStatus status) {
    return switch (status) {
      CardStatusValidSoon() => context.colorScheme.onSurface,
      CardStatusValid() => context.colorScheme.onSurface,
      CardStatusExpiresSoon() => _getStatusWarningColor(context),
      CardStatusExpired() => context.colorScheme.error,
      CardStatusRevoked() => context.colorScheme.error,
      CardStatusCorrupted() => context.colorScheme.error,
      CardStatusUndetermined() => _getStatusWarningColor(context),
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
      CardStatusExpiresSoon() => _getStatusWarningColor(context),
      CardStatusExpired() => context.colorScheme.error,
      CardStatusRevoked() => context.colorScheme.error,
      CardStatusCorrupted() => context.colorScheme.error,
      CardStatusUndetermined() => _getStatusWarningColor(context),
    };
  }

  @override
  Color? backgroundColor(BuildContext context, CardStatus status) => null;

  String _formatIssuer(BuildContext context, Organization issuer) => issuer.displayName.l10nValue(context);

  Color _getStatusWarningColor(BuildContext context) =>
      context.brightness == Brightness.light ? kStatusWarningColorLight : kStatusWarningColorDark;
}
