import 'package:rxdart/rxdart.dart';

import '../../../domain/model/timeline_attribute.dart';
import '../../../domain/model/wallet_card.dart';
import '../wallet_datasource.dart';

class MemoryWalletDataSource implements WalletDataSource {
  final BehaviorSubject<Map<String, WalletCard>> wallet = BehaviorSubject.seeded({});
  final BehaviorSubject<Map<String, List<TimelineAttribute>>> timelineAttributes = BehaviorSubject.seeded({});

  @override
  Future<void> create(WalletCard card) async {
    final cards = wallet.value;
    cards[card.id] = card;
    wallet.add(cards);
  }

  @override
  Future<void> delete(String cardId) async {
    final cards = wallet.value;
    cards.remove(cardId);
    wallet.add(cards);
  }

  @override
  Future<WalletCard?> read(String cardId) async {
    final cards = wallet.value;
    return cards[cardId];
  }

  @override
  Future<List<WalletCard>> readAll() async {
    final cards = wallet.value;
    return cards.values.toList();
  }

  @override
  Future<void> update(WalletCard card) async {
    final cards = wallet.value;
    assert(cards.containsKey(card.id));
    cards[card.id] = card;
    wallet.add(cards);
  }

  @override
  Future<void> createTimelineAttribute(String cardId, TimelineAttribute attribute) async {
    final timelineAttributes = this.timelineAttributes.value;
    if (timelineAttributes[cardId] != null) {
      timelineAttributes[cardId]?.add(attribute);
    } else {
      timelineAttributes[cardId] = [attribute];
    }
    this.timelineAttributes.add(timelineAttributes);
  }

  @override
  Future<List<TimelineAttribute>> readTimelineAttributes(String cardId) async {
    final timelineAttributes = this.timelineAttributes.value;
    return timelineAttributes[cardId] ?? [];
  }

  @override
  Stream<List<WalletCard>> observeCards() => wallet.stream.map((event) => event.values.toList());

  @override
  Stream<List<TimelineAttribute>> observeTimelineAttributes(String cardId) {
    return timelineAttributes.map((event) => event[cardId] ?? []);
  }
}
