import 'package:flutter/material.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../../extension/build_context_extension.dart';
import '../context_mapper.dart';

/// Note: similar to [WalletEventStatusTextMapper] but maps [IssuanceEvent] differently.
class WalletEventStatusTitleMapper extends ContextMapper<WalletEvent, String> {
  WalletEventStatusTitleMapper();

  @override
  String map(BuildContext context, WalletEvent event) {
    return switch (event) {
      DisclosureEvent() => _mapDisclosureEvent(context, event),
      IssuanceEvent() => _mapIssuanceEvent(context, event),
      SignEvent() => _mapSignEvent(context, event),
    };
  }

  String _mapDisclosureEvent(BuildContext context, DisclosureEvent event) {
    return switch (event.type) {
      DisclosureType.regular => _mapRegularDisclosure(context, event),
      DisclosureType.login => _mapLoginDisclosure(context, event),
    };
  }

  String _mapRegularDisclosure(BuildContext context, DisclosureEvent event) {
    return switch (event.status) {
      EventStatus.success => context.l10n.cardHistoryDisclosureSuccess,
      EventStatus.cancelled => context.l10n.cardHistoryDisclosureCancelled,
      EventStatus.error =>
        event.hasSharedAttributes
            ? context.l10n.cardHistoryDisclosureError
            : context.l10n.cardHistoryDisclosureErrorNoDataShared,
    };
  }

  String _mapLoginDisclosure(BuildContext context, DisclosureEvent event) {
    return switch (event.status) {
      EventStatus.success => context.l10n.cardHistoryLoginSuccess,
      EventStatus.cancelled => context.l10n.cardHistoryLoginCancelled,
      EventStatus.error =>
        event.hasSharedAttributes ? context.l10n.cardHistoryLoginError : context.l10n.cardHistoryLoginErrorNoDataShared,
    };
  }

  String _mapIssuanceEvent(BuildContext context, IssuanceEvent event) {
    return switch (event.eventType) {
      IssuanceEventType.cardIssued => '', // No title for successful issuance
      IssuanceEventType.cardRenewed => '', // No title for successful renewal
      IssuanceEventType.cardStatusExpired => context.l10n.historyDetailScreenIssuanceCardExpiredTitle,
      IssuanceEventType.cardStatusRevoked => context.l10n.historyDetailScreenIssuanceCardRevokedTitle,
      IssuanceEventType.cardStatusCorrupted => context.l10n.historyDetailScreenIssuanceCardCorruptedTitle,
    };
  }

  String _mapSignEvent(BuildContext context, SignEvent event) {
    return switch (event.status) {
      EventStatus.success => context.l10n.cardHistorySigningSuccess,
      EventStatus.cancelled => context.l10n.cardHistorySigningCancelled,
      EventStatus.error => context.l10n.cardHistorySigningError,
    };
  }
}
