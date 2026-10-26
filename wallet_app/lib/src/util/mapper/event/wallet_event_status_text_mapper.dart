import 'package:flutter/material.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../../extension/build_context_extension.dart';
import 'wallet_event_status_mapper.dart';

/// Note: similar to [WalletEventStatusTitleMapper] but maps [IssuanceEvent] differently.
class WalletEventStatusTextMapper extends WalletEventStatusMapper {
  WalletEventStatusTextMapper();

  @override
  String mapIssuanceEvent(BuildContext context, IssuanceEvent event) {
    return switch (event.eventType) {
      IssuanceEventType.cardIssued => context.l10n.cardHistoryIssuanceSuccess,
      IssuanceEventType.cardRenewed => context.l10n.cardHistoryTimelineOperationRenewed,
      IssuanceEventType.cardStatusExpired => context.l10n.cardHistoryTimelineOperationExpired,
      IssuanceEventType.cardStatusRevoked => context.l10n.cardHistoryTimelineOperationRevoked,
      IssuanceEventType.cardStatusCorrupted => context.l10n.cardHistoryTimelineOperationCorrupted,
    };
  }
}
