import 'package:rxdart/rxdart.dart';

import '../../../domain/model/wallet_card.dart';
import '../wallet_datasource.dart';

class MemoryWalletDataSource implements WalletDataSource {
  final BehaviorSubject<Map<String, WalletCard>> wallet = BehaviorSubject.seeded({});
  final BehaviorSubject<Map<String, List<String>>> interactions = BehaviorSubject.seeded({});

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
  Future<void> addInteraction(String cardId, String interaction) async {
    final interactions = this.interactions.value;
    interactions[cardId]?.add(interaction);
    this.interactions.add(interactions);
  }

  @override
  Future<List<String>> getInteractions(String cardId) async {
    final interactions = this.interactions.value;
    return interactions[cardId] ?? [];
  }

  @override
  Stream<List<WalletCard>> observeCards() => wallet.stream.map((event) => event.values.toList());

  @override
  Stream<List<String>> observeInteractions(String cardId) => interactions.map((event) => event[cardId] ?? []);
}
