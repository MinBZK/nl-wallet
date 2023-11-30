import 'package:collection/collection.dart';
import 'package:wallet_core/core.dart';

import '../../../domain/model/timeline/timeline_attribute.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../util/mapper/mapper.dart';
import '../../../wallet_core/typed/typed_wallet_core.dart';
import '../wallet_datasource.dart';

class WalletDataSourceImpl implements WalletDataSource {
  final TypedWalletCore _walletCore;
  final Mapper<Card, WalletCard> _cardMapper;

  WalletDataSourceImpl(this._walletCore, this._cardMapper);

  @override
  Future<void> create(WalletCard card) => throw UnimplementedError();

  @override
  Future<List<WalletCard>> readAll() async {
    final cards = await _walletCore.observeCards().first.timeout(const Duration(seconds: 5));
    return _cardMapper.mapList(cards);
  }

  @override
  Future<WalletCard?> read(String cardId) async {
    final cards = await readAll();
    return cards.firstWhereOrNull((element) => element.id == cardId);
  }

  @override
  Future<void> update(WalletCard card) => throw UnimplementedError();

  @override
  Future<void> delete(String cardId) => throw UnimplementedError();

  @override
  Future<void> createTimelineAttribute(TimelineAttribute attribute) => throw UnimplementedError();

  @override
  Stream<List<WalletCard>> observeCards() => _walletCore.observeCards().map((cards) => _cardMapper.mapList(cards));

  @override
  void clear() => throw UnimplementedError();
}
