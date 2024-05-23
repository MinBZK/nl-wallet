import '../../../domain/model/event/wallet_event.dart';

abstract class WalletEventRepository {
  Future<List<WalletEvent>> getEvents();

  Future<List<WalletEvent>> getEventsForCard(String docType);

  Stream<List<WalletEvent>> observeRecentEvents();

  /// Returns most recent [DisclosureEvent] for card filtered by [EventStatus]
  Future<DisclosureEvent?> readMostRecentDisclosureEvent(String docType, EventStatus status);

  /// Returns most recent [IssuanceEvent] for card filtered by [EventStatus]
  Future<IssuanceEvent?> readMostRecentIssuanceEvent(String docType, EventStatus status);
}
