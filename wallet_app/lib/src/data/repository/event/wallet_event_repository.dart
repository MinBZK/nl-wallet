import '../../../domain/model/event/wallet_event.dart';

abstract class WalletEventRepository {
  Future<List<WalletEvent>> getEvents({int? page, int? pageSize, bool removeDuplicatePidEvents = true});

  Future<List<WalletEvent>> getEventsForCard(String attestationId);

  Stream<List<WalletEvent>> observeRecentEvents({bool removeDuplicatePidEvents = true});

  /// Returns most recent [DisclosureEvent] for card filtered by [EventStatus]
  Future<DisclosureEvent?> readMostRecentDisclosureEvent(String attestationId, EventStatus status);

  /// Returns most recent [IssuanceEvent] for card filtered by [EventStatus]
  Future<IssuanceEvent?> readMostRecentIssuanceEvent(String attestationId, EventStatus status);
}
