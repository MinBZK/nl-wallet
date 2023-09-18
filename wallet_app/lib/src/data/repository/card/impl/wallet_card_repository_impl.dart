import 'package:rxdart/rxdart.dart';

import '../../../../domain/model/wallet_card.dart';
import '../../../../util/mapper/card/card_mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../../../../wallet_core/wallet_core.dart';
import '../../../store/active_locale_provider.dart';
import '../wallet_card_repository.dart';

class WalletCardRepositoryImpl implements WalletCardRepository {
  final TypedWalletCore _walletCore;
  final ActiveLocaleProvider _localeProvider;
  final CardMapper _cardMapper;

  WalletCardRepositoryImpl(this._walletCore, this._localeProvider, this._cardMapper);

  List<WalletCard> _mapCards(List<Card> cards, languageCode) {
    return cards.map((card) => _cardMapper.map(card, languageCode)).toList();
  }

  @override
  Stream<List<WalletCard>> observeWalletCards() {
    return CombineLatestStream.combine2(
      _walletCore.observeCards(),
      _localeProvider.observe(),
      (cards, locale) => _mapCards(cards, locale.languageCode),
    );
  }

  @override
  Future<bool> exists(String cardId) async {
    throw UnimplementedError();
  }

  @override
  Future<void> create(WalletCard card) async {
    throw UnimplementedError();
  }

  @override
  Future<List<WalletCard>> readAll() async {
    throw UnimplementedError();
  }

  @override
  Future<WalletCard> read(String cardId) async {
    throw UnimplementedError();
  }

  @override
  Future<void> update(WalletCard card) async {
    throw UnimplementedError();
  }

  @override
  Future<void> delete(String cardId) async {
    throw UnimplementedError();
  }
}
