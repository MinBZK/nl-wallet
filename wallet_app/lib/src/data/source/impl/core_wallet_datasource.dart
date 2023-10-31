import 'package:collection/collection.dart';
import 'package:rxdart/rxdart.dart';

import '../../../domain/model/timeline/timeline_attribute.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../util/mapper/mapper.dart';
import '../../../wallet_core/typed/typed_wallet_core.dart';
import '../../../wallet_core/wallet_core.dart';
import '../../store/active_locale_provider.dart';
import '../wallet_datasource.dart';

class CoreWalletDataSource implements WalletDataSource {
  final TypedWalletCore _walletCore;
  final Mapper<Card, WalletCard> _cardMapper;
  final ActiveLocaleProvider _localeProvider;

  CoreWalletDataSource(this._walletCore, this._cardMapper, this._localeProvider);

  @override
  Future<void> create(WalletCard card) => throw UnimplementedError();

  @override
  Future<List<WalletCard>> readAll() async {
    final cards = await _walletCore.observeCards().first.timeout(const Duration(seconds: 5));
    return cards.map((card) => _cardMapper.map(card)).toList();
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
  Future<List<TimelineAttribute>> readTimelineAttributes() => throw UnimplementedError();

  @override
  Future<List<TimelineAttribute>> readTimelineAttributesByCardId({required String cardId}) =>
      throw UnimplementedError();

  @override
  Future<TimelineAttribute> readTimelineAttributeById({required String timelineAttributeId, String? cardId}) =>
      throw UnimplementedError();

  @override
  Stream<List<WalletCard>> observeCards() => CombineLatestStream.combine2(
        _walletCore.observeCards(),
        _localeProvider.observe(),
        (cards, locale) => cards.map((card) => _cardMapper.map(card)).toList(),
      );

  @override
  void clear() => throw UnimplementedError();
}
