import 'package:flutter/material.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../../extension/build_context_extension.dart';
import '../context_mapper.dart';

/// Maps a [WalletEvent] to a string describing its status.
///
/// The mapping of all events except [IssuanceEvent] is shared, implementations only
/// have to specify how an [IssuanceEvent] should be mapped.
abstract class WalletEventStatusMapper extends ContextMapper<WalletEvent, String> {
  @override
  String map(BuildContext context, WalletEvent input) {
    return switch (input) {
      DeletionEvent() => context.l10n.cardHistoryDeletionSuccess,
      DisclosureEvent() => _mapDisclosureEvent(context, input),
      IssuanceEvent() => mapIssuanceEvent(context, input),
      SignEvent() => _mapSignEvent(context, input),
    };
  }

  String mapIssuanceEvent(BuildContext context, IssuanceEvent event);

  String _mapDisclosureEvent(BuildContext context, DisclosureEvent event) {
    return switch (event.type) {
      DisclosureType.regular => _mapRegularDisclosure(context, event),
      DisclosureType.login => _mapLoginDisclosure(context, event),
    };
  }

  String _mapSignEvent(BuildContext context, SignEvent event) {
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
