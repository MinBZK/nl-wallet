import '../../domain/model/timeline_attribute.dart';
import '../../domain/model/wallet_card.dart';

abstract class WalletDataSource {
  Future<void> create(WalletCard card);

  Future<WalletCard?> read(String cardId);

  Future<void> update(WalletCard card);

  Future<void> delete(String cardId);

  Future<void> createTimelineAttribute(String cardId, TimelineAttribute attribute);

  /// Returns all wallet cards [TimelineAttribute]s sorted by date ASC (oldest first)
  Future<List<TimelineAttribute>> readTimelineAttributes();

  /// Returns all card specific [TimelineAttribute] sorted by date ASC (oldest first)
  Future<List<TimelineAttribute>> readTimelineAttributesByCardId(String cardId);

  Future<List<WalletCard>> readAll();

  Stream<List<WalletCard>> observeCards();

  Stream<List<TimelineAttribute>> observeTimelineAttributes(String cardId);
}
