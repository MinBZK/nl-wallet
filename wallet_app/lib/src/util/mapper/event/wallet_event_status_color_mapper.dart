import 'package:flutter/material.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../../extension/build_context_extension.dart';
import '../context_mapper.dart';

class WalletEventStatusColorMapper extends ContextMapper<WalletEvent, Color> {
  WalletEventStatusColorMapper();

  @override
  Color map(BuildContext context, WalletEvent input) {
    if (useErrorColor(input)) return context.colorScheme.error;
    return context.colorScheme.onSurface;
  }

  bool useErrorColor(WalletEvent input) {
    return switch (input) {
      DisclosureEvent() => input.status != EventStatus.success,
      IssuanceEvent() => _isIssuanceEventError(input.eventType),
      SignEvent() => false,
    };
  }

  bool _isIssuanceEventError(IssuanceEventType eventType) {
    return switch (eventType) {
      IssuanceEventType.cardIssued => false,
      IssuanceEventType.cardRenewed => false,
      IssuanceEventType.cardStatusExpired => true,
      IssuanceEventType.cardStatusCorrupted => true,
      IssuanceEventType.cardStatusRevoked => true,
    };
  }
}
