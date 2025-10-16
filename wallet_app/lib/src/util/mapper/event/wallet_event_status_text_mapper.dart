import 'package:flutter/material.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../../extension/build_context_extension.dart';
import '../context_mapper.dart';

/// Note: similar to [WalletEventStatusTitleMapper] but maps [IssuanceEvent] differently.
class WalletEventStatusTextMapper extends ContextMapper<WalletEvent, String> {
  WalletEventStatusTextMapper();

  @override
  String map(BuildContext context, WalletEvent input) {
    return switch (input) {
      DisclosureEvent() => mapDisclosureEvent(context, input),
      IssuanceEvent() => mapIssuanceEvent(context, input),
      SignEvent() => mapSignEvent(context, input),
    };
  }

  String mapDisclosureEvent(BuildContext context, DisclosureEvent event) {
    return switch (event.type) {
      DisclosureType.regular => _mapRegularDisclosure(context, event),
      DisclosureType.login => _mapLoginDisclosure(context, event),
    };
  }

  String mapIssuanceEvent(BuildContext context, IssuanceEvent event) {
    return switch (event.eventType) {
      IssuanceEventType.cardIssued => context.l10n.cardHistoryIssuanceSuccess,
      IssuanceEventType.cardRenewed => context.l10n.cardHistoryTimelineOperationRenewed,
      IssuanceEventType.cardStatusExpired => context.l10n.cardHistoryTimelineOperationExpired,
      IssuanceEventType.cardStatusCorrupted => context.l10n.cardHistoryTimelineOperationCorrupted,
      IssuanceEventType.cardStatusRevoked => context.l10n.cardHistoryTimelineOperationRevoked,
    };
  }

  String mapSignEvent(BuildContext context, SignEvent event) {
    return switch (event.status) {
      EventStatus.success => context.l10n.cardHistorySigningSuccess,
      EventStatus.cancelled => context.l10n.cardHistorySigningCancelled,
      EventStatus.error => context.l10n.cardHistorySigningError,
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
}
