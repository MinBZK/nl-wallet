import 'package:collection/collection.dart';

import '../../domain/model/card/wallet_card.dart';
import '../../domain/model/configuration/flutter_app_configuration.dart';
import '../../domain/model/event/wallet_event.dart';
import '../extension/pid_attestation_extension.dart';

typedef AppConfigurationProvider = Future<FlutterAppConfiguration> Function();

/// Mixin providing PID filtering logic based on the app configuration.
mixin PidFilterMixin {
  // Used to provide the mixin with the [FlutterAppConfiguration]
  AppConfigurationProvider get configProvider;

  Future<List<WalletCard>> filterDuplicatePidCards(List<WalletCard> cards) async {
    final config = await configProvider();

    final pidAttestationTypes = config.pidAttestationTypes;

    // Find a matching pid (ordering of [pidAttestations] is leading)
    WalletCard? pidToMaintain;
    for (final pidAttestation in config.pidAttestations) {
      pidToMaintain = cards.firstWhereOrNull(pidAttestation.matches);
      if (pidToMaintain != null) break;
    }

    // Early return in case we don't need to filter
    if (pidToMaintain == null) return cards.toList();

    // Only maintain selected PID + non-PID cards
    return cards.where((card) {
      if (card == pidToMaintain) return true;
      return !pidAttestationTypes.contains(card.attestationType);
    }).toList();
  }

  /// Filters out duplicate events for PID cards, keeping only the event for the
  /// highest priority PID attestation as defined in the app configuration.
  Future<List<WalletEvent>> filterDuplicatePidEvents(List<WalletEvent> events) async {
    final config = await configProvider();
    final pidAttestationTypes = config.pidAttestationTypes;
    final pidEvents = events.where((it) => pidAttestationTypes.contains(_getCard(it)?.attestationType));

    final result = <WalletEvent>[];
    final handled = <WalletEvent>{};

    for (final event in events) {
      // (Related) event already processed, skip.
      if (handled.contains(event)) continue;

      // Check if card is a PID card, if not, simply add.
      final isPidCard = pidAttestationTypes.contains(_getCard(event)?.attestationType);
      if (!isPidCard) {
        result.add(event);
        continue;
      }

      // Find related events (e.g. duplicate pid renew events).
      final relatedEvents = pidEvents.where((it) => it.matches(event)).toList();

      // Find the event with highest priority based on config, and add it.
      final priorityEvent = _findPriorityEvent(relatedEvents, config);
      if (priorityEvent != null) result.add(priorityEvent);

      // Mark related events as handled, so they are not processed again.
      handled.addAll(relatedEvents);
    }
    return result;
  }

  /// Extracts the [WalletCard] associated with an event, if applicable.
  WalletCard? _getCard(WalletEvent event) => switch (event) {
    IssuanceEvent(:final card) || DeletionEvent(:final card) => card,
    _ => null,
  };

  /// Selects the event corresponding to the preferred PID attestation type
  /// based on the priority order defined in [config].
  WalletEvent? _findPriorityEvent(List<WalletEvent> events, FlutterAppConfiguration config) {
    for (final pidAttestation in config.pidAttestations) {
      final match = events.firstWhereOrNull((e) {
        final card = _getCard(e);
        return card != null && pidAttestation.matches(card);
      });
      if (match != null) return match;
    }
    return events.firstOrNull;
  }
}

extension _WalletEventExtensions on WalletEvent {
  /// Checks if two (Issuance/Deletion) events refer to the same logical action (i.e. same type, status, and time)
  bool matches(WalletEvent other) {
    if (this == other) return true;
    if (runtimeType != other.runtimeType) return false;
    final self = this;
    return switch ((self, other)) {
      (final IssuanceEvent s, final IssuanceEvent o) =>
        s.eventType == o.eventType && s.status == o.status && s.dateTime == o.dateTime,
      (final DeletionEvent s, final DeletionEvent o) => s.status == o.status && s.dateTime == o.dateTime,
      _ => false,
    };
  }
}
