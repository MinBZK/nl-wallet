import '../../domain/model/timeline_attribute.dart';
import '../../domain/model/wallet_card.dart';

abstract class WalletDataSource {
  Future<void> create(WalletCard card);

  Future<WalletCard?> read(String cardId);

  Future<void> update(WalletCard card);

  Future<void> delete(String cardId);

  Future<void> createTimelineAttribute(String cardId, TimelineAttribute attribute);

  Future<List<TimelineAttribute>> readTimelineAttributes(String cardId);

  Future<List<WalletCard>> readAll();

  Stream<List<WalletCard>> observeCards();

  Stream<List<TimelineAttribute>> observeTimelineAttributes(String cardId);
}
