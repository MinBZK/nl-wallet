import '../../../domain/model/event/wallet_event.dart';

abstract class WalletEventRepository {
  Future<List<WalletEvent>> getEvents();

  Future<List<WalletEvent>> getEventsForCard(String attestationId);

  Stream<List<WalletEvent>> observeRecentEvents();

  /// Returns most recent [DisclosureEvent] for card filtered by [EventStatus]
  Future<DisclosureEvent?> readMostRecentDisclosureEvent(String attestationId, EventStatus status);

  /// Returns most recent [IssuanceEvent] for card filtered by [EventStatus]
  Future<IssuanceEvent?> readMostRecentIssuanceEvent(String attestationId, EventStatus status);
}
