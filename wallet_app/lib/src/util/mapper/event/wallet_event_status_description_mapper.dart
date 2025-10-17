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
      DisclosureEvent() => _mapDisclosureEvent(context, input),
      IssuanceEvent() => _mapIssuanceEvent(context, input),
      SignEvent() => _mapSignEvent(context, input),
    };
  }

  String _mapDisclosureEvent(BuildContext context, DisclosureEvent event) {
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
      DisclosureType.regular =>
        event.hasSharedAttributes
            ? context.l10n.historyDetailScreenDisclosureErrorDescription(organizationName)
            : context.l10n.historyDetailScreenDisclosureErrorNoDataSharedDescription(organizationName),
      DisclosureType.login =>
        event.hasSharedAttributes
            ? context.l10n.historyDetailScreenLoginErrorDescription(organizationName)
            : context.l10n.historyDetailScreenLoginErrorNoDataSharedDescription(organizationName),
    };
  }

  String _mapCancelledDisclosureEvent(BuildContext context, DisclosureEvent event) {
    final organizationName = event.relyingParty.displayName.l10nValue(context);
    return switch (event.type) {
      DisclosureType.regular => context.l10n.historyDetailScreenDisclosureCancelledDescription(organizationName),
      DisclosureType.login => context.l10n.historyDetailScreenLoginCancelledDescription(organizationName),
    };
  }

  String _mapIssuanceEvent(BuildContext context, IssuanceEvent event) {
    return switch (event.eventType) {
      IssuanceEventType.cardIssued => '', // No description for successful issuance
      IssuanceEventType.cardRenewed => '', // No description for successful renewal
      IssuanceEventType.cardStatusExpired => context.l10n.historyDetailScreenIssuanceCardExpiredDescription,
      IssuanceEventType.cardStatusRevoked => context.l10n.historyDetailScreenIssuanceCardRevokedDescription,
      IssuanceEventType.cardStatusCorrupted => context.l10n.historyDetailScreenIssuanceCardCorruptedDescription,
    };
  }

  String _mapSignEvent(BuildContext context, SignEvent event) {
    if (event.status == EventStatus.cancelled) {
      return context.l10n.historyDetailScreenSigningStatusRejectedDescription(
        event.relyingParty.displayName.l10nValue(context),
      );
    }
    return '';
  }
}
