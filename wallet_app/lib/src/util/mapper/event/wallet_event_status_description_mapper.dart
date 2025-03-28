import 'package:flutter/material.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../../extension/build_context_extension.dart';
import '../../extension/localized_text_extension.dart';
import '../context_mapper.dart';

class WalletEventStatusDescriptionMapper extends ContextMapper<WalletEvent, String> {
  WalletEventStatusDescriptionMapper();

  @override
  String map(BuildContext context, WalletEvent input) {
    return switch (input) {
      DisclosureEvent() => mapDisclosureEvent(context, input),
      IssuanceEvent() => mapIssuanceEvent(context, input),
      SignEvent() => mapSignEvent(context, input),
    };
  }

  String mapDisclosureEvent(BuildContext context, DisclosureEvent event) {
    switch (event.status) {
      case EventStatus.success:
        return '';
      case EventStatus.cancelled:
        return _mapCancelledDisclosureEvent(context, event);
      case EventStatus.error:
        return _mapErrorDisclosureEvent(context, event);
    }
  }

  String _mapErrorDisclosureEvent(BuildContext context, DisclosureEvent event) {
    final organizationName = event.relyingParty.displayName.l10nValue(context);
    return switch (event.type) {
      DisclosureType.regular => event.hasSharedAttributes
          ? context.l10n.historyDetailScreenDisclosureErrorDescription(organizationName)
          : context.l10n.historyDetailScreenDisclosureErrorNoDataSharedDescription(organizationName),
      DisclosureType.login => event.hasSharedAttributes
          ? context.l10n.historyDetailScreenLoginErrorDescription(organizationName)
          : context.l10n.historyDetailScreenLoginErrorNoDataSharedDescription(organizationName)
    };
  }

  String _mapCancelledDisclosureEvent(BuildContext context, DisclosureEvent event) {
    final organizationName = event.relyingParty.displayName.l10nValue(context);
    return switch (event.type) {
      DisclosureType.regular => context.l10n.historyDetailScreenDisclosureCancelledDescription(organizationName),
      DisclosureType.login => context.l10n.historyDetailScreenLoginCancelledDescription(organizationName)
    };
  }

  String mapIssuanceEvent(BuildContext context, IssuanceEvent event) {
    return context.l10n.historyDetailScreenIssuanceSuccessDescription;
    // In the future, I imagine we re-introduce renewal/expiry through separate events.
    // For reference keeping the correct translations here:
    // renewal --> context.l10n.historyDetailScreenOperationStatusRenewedDescription;
    // expiry --> context.l10n.historyDetailScreenOperationStatusExpiredDescription;
  }

  String mapSignEvent(BuildContext context, SignEvent event) {
    if (event.status == EventStatus.cancelled) {
      return context.l10n.historyDetailScreenSigningStatusRejectedDescription(
        event.relyingParty.displayName.l10nValue(context),
      );
    }
    return '';
  }
}
