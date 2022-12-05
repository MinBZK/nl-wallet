import 'package:collection/collection.dart';
import 'package:rxdart/rxdart.dart';

import '../../../domain/model/timeline_attribute.dart';
import '../../../domain/model/wallet_card.dart';
import '../wallet_datasource.dart';

class MemoryWalletDataSource implements WalletDataSource {
  final BehaviorSubject<List<WalletCard>> wallet = BehaviorSubject.seeded([]);
  final BehaviorSubject<Map<String, List<TimelineAttribute>>> timelineAttributes = BehaviorSubject.seeded({});

  @override
  Future<void> create(WalletCard card) async {
    final cards = wallet.value;
    delete(card.id); // Check to prevent duplicate cards entries
    cards.add(card);
    wallet.add(cards);
  }

  @override
  Future<WalletCard?> read(String cardId) async {
    final cards = wallet.value;
    return cards.firstWhereOrNull((element) => element.id == cardId);
  }

  @override
  Future<void> update(WalletCard card) async {
    final cards = wallet.value;
    assert(cards.firstWhereOrNull((element) => element.id == card.id) != null);
    cards[cards.indexWhere((element) => element.id == card.id)] = card;
    wallet.add(cards);
  }

  @override
  Future<void> delete(String cardId) async {
    final cards = wallet.value;
    cards.removeWhere((element) => element.id == cardId);
    wallet.add(cards);
  }

  @override
  Future<List<WalletCard>> readAll() async {
    return wallet.value;
  }

  @override
  Future<void> createTimelineAttribute(String cardId, TimelineAttribute attribute) async {
    final attributes = timelineAttributes.value;
    if (attributes[cardId] != null) {
      attributes[cardId]?.add(attribute);
    } else {
      attributes[cardId] = [attribute];
    }
    timelineAttributes.add(attributes);
  }

  /// Returns all wallet cards [TimelineAttribute]s sorted by date ASC (oldest first)
  @override
  Future<List<TimelineAttribute>> readTimelineAttributes() async {
    List<TimelineAttribute> attributes = [];

    // Collect all cards; timeline attributes
    timelineAttributes.value.forEach((key, value) {
      attributes.addAll(value);
    });

    // Sort by date/time
    attributes.sortBy((element) => element.dateTime);

    return attributes;
  }

  /// Returns all card specific [TimelineAttribute] sorted by date ASC (oldest first)
  @override
  Future<List<TimelineAttribute>> readTimelineAttributesByCardId(String cardId) async {
    List<TimelineAttribute> attributes = timelineAttributes.value[cardId] ?? [];
    attributes.sortBy((element) => element.dateTime); // Sort by date/time
    return attributes;
  }

  @override
  Stream<List<WalletCard>> observeCards() => wallet.stream;

  @override
  Stream<List<TimelineAttribute>> observeTimelineAttributes(String cardId) {
    return timelineAttributes.map((event) => event[cardId] ?? []);
  }
}
