import 'package:collection/collection.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/configuration/flutter_app_configuration.dart';
import '../../../../domain/model/event/wallet_event.dart';
import '../../../../util/extension/pid_attestation_extension.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../wallet_event_repository.dart';

class CoreWalletEventRepository extends WalletEventRepository {
  final TypedWalletCore _walletCore;
  final Mapper<core.WalletEvent, WalletEvent> _walletEventMapper;
  final Mapper<core.FlutterConfiguration, FlutterAppConfiguration> _flutterAppConfigurationMapper;

  CoreWalletEventRepository(this._walletCore, this._walletEventMapper, this._flutterAppConfigurationMapper);

  @override
  Future<List<WalletEvent>> getEvents({bool removeDuplicatePidEvents = true}) async {
    final coreEvents = await _walletCore.getHistory();
    final events = _walletEventMapper.mapList(coreEvents);
    if (!removeDuplicatePidEvents) return events;
    return filterDuplicatePidEvents(events);
  }

  @override
  Future<List<WalletEvent>> getEventsForCard(String attestationId) async {
    final coreEvents = await _walletCore.getHistoryForCard(attestationId);
    return _walletEventMapper.mapList(coreEvents);
  }

  @override
  Future<DisclosureEvent?> readMostRecentDisclosureEvent(String attestationId, EventStatus status) async {
    return _walletEventMapper
        .mapList(await _walletCore.getHistoryForCard(attestationId))
        .whereType<DisclosureEvent>()
        .firstWhereOrNull((e) => e.status == status);
  }

  @override
  Future<IssuanceEvent?> readMostRecentIssuanceEvent(String attestationId, EventStatus status) async {
    return _walletEventMapper
        .mapList(await _walletCore.getHistoryForCard(attestationId))
        .whereType<IssuanceEvent>()
        .firstWhereOrNull((e) => e.status == status);
  }

  @override
  Stream<List<WalletEvent>> observeRecentEvents({bool removeDuplicatePidEvents = true}) {
    final recentEventsStream = _walletCore.observeRecentHistory().map(_walletEventMapper.mapList);
    if (!removeDuplicatePidEvents) return recentEventsStream;
    return recentEventsStream.asyncMap(filterDuplicatePidEvents);
  }

  /// Filters out duplicate events for PID cards, keeping only the event for the
  /// highest priority PID attestation as defined in the app configuration.
  Future<List<WalletEvent>> filterDuplicatePidEvents(List<WalletEvent> events) async {
    final config = await _walletCore.observeConfig().map(_flutterAppConfigurationMapper.map).first;
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
  /// Checks if two (Issuance/Deletion) events refer to the same logical action (e.g. same type, status, and time)
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
