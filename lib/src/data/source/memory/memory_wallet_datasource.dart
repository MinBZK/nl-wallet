import 'package:collection/collection.dart';
import 'package:rxdart/rxdart.dart';

import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/timeline/timeline_attribute.dart';
import '../../../domain/model/wallet_card.dart';
import '../wallet_datasource.dart';

class MemoryWalletDataSource implements WalletDataSource {
  final BehaviorSubject<List<WalletCard>> walletCards = BehaviorSubject.seeded([]);
  final BehaviorSubject<List<TimelineAttribute>> timelineAttributes = BehaviorSubject.seeded([]);

  @override
  Future<void> create(WalletCard card) async {
    final cards = walletCards.value;
    delete(card.id); // Check to prevent duplicate cards entries
    cards.add(card);
    walletCards.add(cards);
  }

  @override
  Future<List<WalletCard>> readAll() async {
    return walletCards.value;
  }

  @override
  Future<WalletCard?> read(String cardId) async {
    final cards = walletCards.value;
    return cards.firstWhereOrNull((element) => element.id == cardId);
  }

  @override
  Future<void> update(WalletCard card) async {
    final cards = walletCards.value;
    assert(cards.firstWhereOrNull((element) => element.id == card.id) != null);
    cards[cards.indexWhere((element) => element.id == card.id)] = card;
    walletCards.add(cards);
  }

  @override
  Future<void> delete(String cardId) async {
    final cards = walletCards.value;
    cards.removeWhere((element) => element.id == cardId);
    walletCards.add(cards);
  }

  @override
  Future<void> createTimelineAttribute(TimelineAttribute attribute) async {
    timelineAttributes.value.add(attribute);
  }

  /// Returns all wallet cards [TimelineAttribute]s sorted by date ASC (oldest first)
  @override
  Future<List<TimelineAttribute>> readTimelineAttributes() async {
    List<TimelineAttribute> attributes = _getAllTimelineAttributes();
    attributes.sortBy((element) => element.dateTime);
    return attributes;
  }

  /// Returns [TimelineAttribute] with card specific data that has been used, sorted by date ASC (oldest first)
  @override
  Future<List<TimelineAttribute>> readTimelineAttributesByCardId({required String cardId}) async {
    // Filter [TimelineAttribute]s containing [cardId]
    List<TimelineAttribute> results = timelineAttributes.value.where((timelineAttribute) {
      return timelineAttribute.dataAttributes.firstWhereOrNull((dataAttribute) {
            return dataAttribute.sourceCardId == cardId;
          }) !=
          null;
    }).toList();

    // Only show [DataAttribute]s from [cardId]
    results = results.map((timelineAttribute) {
      List<DataAttribute> filtered = _filterDataAttributesByCardId(cardId, timelineAttribute.dataAttributes);
      return timelineAttribute.copyWith(dataAttributes: filtered);
    }).toList();

    // Sort by date/time
    results.sortBy((element) => element.dateTime);
    return results;
  }

  /// Returns single [TimelineAttribute] by [timelineAttributeId]
  @override
  Future<TimelineAttribute> readTimelineAttributeById({required String timelineAttributeId, String? cardId}) async {
    TimelineAttribute timelineAttribute = _getAllTimelineAttributes().firstWhere((element) {
      return element.id == timelineAttributeId;
    });

    final List<DataAttribute> filtered = _filterDataAttributesByCardId(cardId, timelineAttribute.dataAttributes);
    return timelineAttribute.copyWith(dataAttributes: filtered);
  }

  @override
  Stream<List<WalletCard>> observeCards() => walletCards.stream;

  List<DataAttribute> _filterDataAttributesByCardId(String? cardId, List<DataAttribute> dataAttributes) {
    if (cardId == null) return dataAttributes;
    return dataAttributes.where((element) => element.sourceCardId == cardId).toList();
  }

  List<TimelineAttribute> _getAllTimelineAttributes() {
    return timelineAttributes.value;
  }
}
