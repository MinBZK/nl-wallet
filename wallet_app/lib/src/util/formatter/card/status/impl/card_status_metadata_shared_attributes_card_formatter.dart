import 'package:flutter/material.dart';

import '../../../../../domain/model/card/status/card_status.dart';
import '../../../../../domain/model/card/wallet_card.dart';
import '../../../../extension/build_context_extension.dart';
import '../card_status_metadata_formatter.dart';

class CardStatusMetadataSharedAttributesCardFormatter implements CardStatusMetadataFormatter {
  @override
  bool show(CardStatus status) {
    return switch (status) {
      CardStatusValidSoon() => false,
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
    return switch (card.status) {
      CardStatusValidSoon() => '',
      CardStatusValid() => '',
      CardStatusExpiresSoon() => '',
      CardStatusExpired() => context.l10n.cardStatusMetadataSharedAttributesCardExpired,
      CardStatusRevoked() => context.l10n.cardStatusMetadataSharedAttributesCardRevoked,
      CardStatusCorrupted() => context.l10n.cardStatusMetadataSharedAttributesCardCorrupted,
      CardStatusUndetermined() => context.l10n.cardStatusMetadataSharedAttributesCardUndetermined,
    };
  }

  @override
  Color textColor(BuildContext context, CardStatus status) => context.colorScheme.onSurface;

  @override
  IconData? icon(CardStatus status) {
    return switch (status) {
      CardStatusValidSoon() => null,
      CardStatusValid() => null,
      CardStatusExpiresSoon() => null,
      CardStatusExpired() => Icons.warning_amber,
      CardStatusRevoked() => Icons.warning_amber,
      CardStatusCorrupted() => Icons.warning_amber,
      CardStatusUndetermined() => Icons.feedback_outlined,
    };
  }

  @override
  Color? iconColor(BuildContext context, CardStatus status) => context.colorScheme.onSurfaceVariant;

  @override
  Color? backgroundColor(BuildContext context, CardStatus status) => null;
}
