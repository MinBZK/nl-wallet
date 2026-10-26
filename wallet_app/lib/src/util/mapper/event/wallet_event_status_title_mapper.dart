import 'package:flutter/material.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../../extension/build_context_extension.dart';
import 'wallet_event_status_mapper.dart';

/// Note: similar to [WalletEventStatusTextMapper] but maps [IssuanceEvent] differently.
class WalletEventStatusTitleMapper extends WalletEventStatusMapper {
  WalletEventStatusTitleMapper();

  @override
  String mapIssuanceEvent(BuildContext context, IssuanceEvent event) {
    return switch (event.eventType) {
      IssuanceEventType.cardIssued => '', // No title for successful issuance
      IssuanceEventType.cardRenewed => '', // No title for successful renewal
      IssuanceEventType.cardStatusExpired => context.l10n.historyDetailScreenIssuanceCardExpiredTitle,
      IssuanceEventType.cardStatusRevoked => context.l10n.historyDetailScreenIssuanceCardRevokedTitle,
      IssuanceEventType.cardStatusCorrupted => context.l10n.historyDetailScreenIssuanceCardCorruptedTitle,
    };
  }
}
