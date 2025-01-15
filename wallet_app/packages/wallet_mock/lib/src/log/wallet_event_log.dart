import 'package:rxdart/rxdart.dart';
import 'package:wallet_core/core.dart';

import '../util/extension/string_extension.dart';

class WalletEventLog {
  final List<WalletEvent> _log = List.empty(growable: true);
  final BehaviorSubject<List<WalletEvent>> _logSubject = BehaviorSubject.seeded([]);

  WalletEventLog();

  List<WalletEvent> get log => List.from(_log);

  Stream<List<WalletEvent>> get logStream => _logSubject.stream;

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
    final bool isLogin = disclosure.mapOrNull(request: (request) => request.requestedCards.onlyDisclosesBsn) ?? false;
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
          /// We invent a policy here, mainly because it's only for the mock and not used in the current setup.
          final relyingParty = requestAttributesMissing.relyingParty;
          return RequestPolicy(
            dataSharedWithThirdParties: false,
            dataDeletionPossible: false,
            policyUrl: relyingParty.privacyPolicyUrl ?? relyingParty.webUrl ?? '',
          );
        },
      ),
      status: status,
      typ: isLogin ? DisclosureType.Login : DisclosureType.Regular,
    );
    _logEvent(event);
  }

  /// Log the moment where attributes are disclosed as part of the sign/issuance process
  void logDisclosureStep(
    Organization organization,
    RequestPolicy policy,
    List<DisclosureCard> requestedCards,
    DisclosureStatus status, {
    List<LocalizedString>? purpose,
  }) {
    final event = WalletEvent.disclosure(
      dateTime: DateTime.now().toIso8601String(),
      relyingParty: organization,
      purpose: purpose ?? ''.untranslated,
      requestedCards: requestedCards,
      requestPolicy: policy,
      status: status,
      typ: requestedCards.onlyDisclosesBsn ? DisclosureType.Login : DisclosureType.Regular,
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
    _logSubject.add(_log);
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

extension _DisclosureCardsExtension on List<DisclosureCard> {
  bool get onlyDisclosesBsn {
    return length == 1 && first.attributes.length == 1 && first.attributes.first.key == 'mock.citizenshipNumber';
  }
}
