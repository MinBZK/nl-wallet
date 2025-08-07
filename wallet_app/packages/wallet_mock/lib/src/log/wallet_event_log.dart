import 'package:rxdart/rxdart.dart';
import 'package:wallet_core/core.dart';

import '../util/extension/string_extension.dart';

class WalletEventLog {
  final List<WalletEvent> _log = List.empty(growable: true);
  final BehaviorSubject<List<WalletEvent>> _logSubject = BehaviorSubject.seeded([]);

  WalletEventLog();

  List<WalletEvent> get log => List.from(_log);

  Stream<List<WalletEvent>> get logStream => _logSubject.stream;

  bool _eventContainsCardWithAttestationId(WalletEvent_Disclosure disclosure, String attestationId) {
    if (disclosure.sharedAttestations == null) return false;

    /// Check if the provided attestationId was used in this request
    return disclosure.sharedAttestations!.any(
      (card) =>
          switch (card.identity) {
            AttestationIdentity_Ephemeral() => '',
            AttestationIdentity_Fixed(:final id) => id,
          } ==
          attestationId,
    );
  }

  List<WalletEvent> logForAttestationId(String attestationId) => log
      .where(
        (event) => switch (event) {
          WalletEvent_Disclosure() => _eventContainsCardWithAttestationId(event, attestationId),
          WalletEvent_Issuance() => event.attestation.attestationType == attestationId,
        },
      )
      .toList();

  void logDisclosure(StartDisclosureResult disclosure, DisclosureStatus status) {
    final List<AttestationPresentation> sharedAttestations = switch (disclosure) {
      StartDisclosureResult_Request(:final requestedAttestations) => requestedAttestations,
      StartDisclosureResult_RequestAttributesMissing() => [],
    };
    final RequestPolicy policy = switch (disclosure) {
      StartDisclosureResult_Request(:final policy) => policy,
      StartDisclosureResult_RequestAttributesMissing(:final relyingParty) => RequestPolicy(
          dataSharedWithThirdParties: false,
          dataDeletionPossible: false,
          policyUrl: relyingParty.privacyPolicyUrl ?? relyingParty.webUrl ?? '',
        ) /* We invent a policy here, mainly because it's only for the mock and not used in the current setup. */,
    };
    final bool isLogin = sharedAttestations.onlyContainsBsn;
    final event = WalletEvent.disclosure(
      id: 'id123',
      dateTime: DateTime.now().toIso8601String(),
      relyingParty: disclosure.relyingParty,
      purpose: disclosure.requestPurpose,
      sharedAttestations: sharedAttestations,
      requestPolicy: policy,
      status: status,
      typ: isLogin ? DisclosureType.Login : DisclosureType.Regular,
    );
    _logEvent(event);
  }

  /// Log the moment where attributes are disclosed as part of the sign/issuance process
  void logDisclosureStep(
    Organization organization,
    RequestPolicy policy,
    List<AttestationPresentation> sharedAttestations,
    DisclosureStatus status, {
    List<LocalizedString>? purpose,
  }) {
    final event = WalletEvent.disclosure(
      id: 'id123',
      dateTime: DateTime.now().toIso8601String(),
      relyingParty: organization,
      purpose: purpose ?? ''.untranslated,
      sharedAttestations: sharedAttestations,
      requestPolicy: policy,
      status: status,
      typ: sharedAttestations.onlyContainsBsn ? DisclosureType.Login : DisclosureType.Regular,
    );
    _logEvent(event);
  }

  void logIssuance(AttestationPresentation attestation, {bool isRenewal = false}) {
    final event = WalletEvent.issuance(
      id: 'id123',
      dateTime: DateTime.now().toIso8601String(),
      attestation: attestation,
      renewed: isRenewal,
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
      (event) => switch (event) {
        WalletEvent_Disclosure(:final relyingParty) => relyingParty == organization,
        WalletEvent_Issuance(:final attestation) => attestation.issuer == organization,
      },
    );
  }

  void reset() => _log.clear();
}

extension on List<AttestationPresentation> {
  bool get onlyContainsBsn {
    return length == 1 && first.attributes.length == 1 && first.attributes.first.key == 'mock_citizenshipNumber';
  }
}
