import '../../domain/model/timeline/timeline_attribute.dart';
import '../../domain/model/wallet_card.dart';

abstract class WalletDataSource {
  Future<void> create(WalletCard card);

  Future<List<WalletCard>> readAll();

  Future<WalletCard?> read(String cardId);

  Future<void> update(WalletCard card);

  Future<void> delete(String cardId);

  Future<void> createTimelineAttribute(TimelineAttribute attribute);

  Stream<List<WalletCard>> observeCards();

  /// Removes all in-memory data; both [WalletCard]s and [TimelineAttribute]s
  void clear();
}
