import 'package:flutter/material.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../../extension/build_context_extension.dart';
import '../../extension/localized_text_extension.dart';
import '../context_mapper.dart';

/// Note: similar to [WalletEventStatusTextMapper] but maps [IssuanceEvent] differently.
class WalletEventStatusTitleMapper extends ContextMapper<WalletEvent, String> {
  WalletEventStatusTitleMapper();

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
      DisclosureType.regular => mapRegularDisclosure(context, event.status),
      DisclosureType.login => mapLoginDisclosure(context, event.status),
    };
  }

  String mapRegularDisclosure(BuildContext context, EventStatus status) {
    return switch (status) {
      EventStatus.success => context.l10n.cardHistoryDisclosureSuccess,
      EventStatus.cancelled => context.l10n.cardHistoryDisclosureCancelled,
      EventStatus.error => context.l10n.cardHistoryDisclosureError,
    };
  }

  String mapLoginDisclosure(BuildContext context, EventStatus status) {
    return switch (status) {
      EventStatus.success => context.l10n.cardHistoryLoginSuccess,
      EventStatus.cancelled => context.l10n.cardHistoryLoginCancelled,
      EventStatus.error => context.l10n.cardHistoryLoginError,
    };
  }

  String mapIssuanceEvent(BuildContext context, IssuanceEvent input) => input.card.title.l10nValue(context);

  String mapSignEvent(BuildContext context, SignEvent event) {
    return switch (event.status) {
      EventStatus.success => context.l10n.cardHistorySigningSuccess,
      EventStatus.cancelled => context.l10n.cardHistorySigningCancelled,
      EventStatus.error => context.l10n.cardHistorySigningError,
    };
  }
}
