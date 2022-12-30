import '../../domain/model/timeline/timeline_attribute.dart';
import '../../domain/model/wallet_card.dart';

abstract class WalletDataSource {
  Future<void> create(WalletCard card);

  Future<List<WalletCard>> readAll();

  Future<WalletCard?> read(String cardId);

  Future<void> update(WalletCard card);

  Future<void> delete(String cardId);

  Future<void> createTimelineAttribute(TimelineAttribute attribute);

  /// Returns all wallet cards [TimelineAttribute]s sorted by date ASC (oldest first)
  Future<List<TimelineAttribute>> readTimelineAttributes();

  /// Returns all card specific [TimelineAttribute] sorted by date ASC (oldest first)
  Future<List<TimelineAttribute>> readTimelineAttributesByCardId({required String cardId});

  /// Returns single [TimelineAttribute] by [timelineAttributeId]
  Future<TimelineAttribute> readTimelineAttributeById({required String timelineAttributeId, String? cardId});

  Stream<List<WalletCard>> observeCards();
}
