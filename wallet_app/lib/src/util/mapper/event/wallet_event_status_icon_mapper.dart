import 'package:flutter/material.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../mapper.dart';

class WalletEventStatusIconMapper extends Mapper<WalletEvent, IconData?> {
  WalletEventStatusIconMapper();

  @override
  IconData? map(WalletEvent input) {
    return switch (input) {
      DisclosureEvent() => _mapEventStatus(input.status),
      IssuanceEvent() => _mapIssuanceEventType(input.eventType),
      SignEvent() => _mapEventStatus(input.status),
    };
  }

  IconData? _mapEventStatus(EventStatus eventStatus) {
    return switch (eventStatus) {
      EventStatus.success => null,
      EventStatus.cancelled => Icons.block_flipped,
      EventStatus.error => Icons.error_outline_outlined,
    };
  }

  IconData? _mapIssuanceEventType(IssuanceEventType eventType) {
    return switch (eventType) {
      IssuanceEventType.cardIssued => null,
      IssuanceEventType.cardRenewed => null,
      IssuanceEventType.cardStatusExpired => Icons.event_busy,
      IssuanceEventType.cardStatusCorrupted => Icons.block_flipped,
      IssuanceEventType.cardStatusRevoked => Icons.close,
    };
  }
}
