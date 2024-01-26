import 'package:wallet_core/core.dart';

import '../data/mock/mock_organizations.dart';
import '../util/extension/string_extension.dart';

class WalletEventLog {
  final List<WalletEvent> _log = List.empty(growable: true);

  WalletEventLog();

  List<WalletEvent> get log => List.from(_log);

  List<WalletEvent> logForDocType(String docType) => log
      .where(
        (event) => event.map(
          disclosure: (WalletEvent_Disclosure disclosure) {
            if (disclosure.requestedCards == null) return false;

            /// Check if the provided docType was used in this request
            return disclosure.requestedCards!.any((card) => card.docType == docType);
          },
          issuance: (WalletEvent_Issuance issuance) => issuance.card.docType == docType,
        ),
      )
      .toList();

  void logDisclosure(StartDisclosureResult disclosure, DisclosureStatus status) {
    final event = WalletEvent.disclosure(
      dateTime: DateTime.now().toIso8601String(),
      relyingParty: disclosure.relyingParty,
      purpose: disclosure.requestPurpose,
      requestedCards: disclosure.map(
        request: (request) => request.requestedCards,
        requestAttributesMissing: (requestAttributesMissing) => [],
      ),
      requestPolicy: disclosure.map(
        request: (request) => request.policy,
        requestAttributesMissing: (requestAttributesMissing) {
          // FIXME: Make nullable or resolve from RequestedAttributesMissing model
          return RequestPolicy(
            dataSharedWithThirdParties: false,
            dataDeletionPossible: false,
            policyUrl: 'https://example.org',
          );
        },
      ),
      status: status,
    );
    _logEvent(event);
  }

  void logDisclosureStep(
      Organization organization, RequestPolicy policy, List<DisclosureCard> requestedCards, DisclosureStatus status) {
    final event = WalletEvent.disclosure(
      dateTime: DateTime.now().toIso8601String(),
      relyingParty: organization,
      purpose: ''.untranslated,
      requestedCards: requestedCards,
      requestPolicy: policy,
      status: status,
    );
    _logEvent(event);
  }

  void logIssuance(Card card) {
    final event = WalletEvent.issuance(
      dateTime: DateTime.now().toIso8601String(),
      card: card,
    );
    _logEvent(event);
  }

  void _logEvent(WalletEvent event) {
    _log.add(event);
    _log.sort((a, b) => b.dateTime.compareTo(a.dateTime));
  }

  bool includesInteractionWith(Organization organization) {
    return _log.any(
      (event) {
        return event.map(
          disclosure: (disclosure) {
            return disclosure.relyingParty == organization;
          },
          issuance: (issuance) {
            return issuance.card.issuer == organization;
          },
        );
      },
    );
  }

  void reset() => _log.clear();
}
