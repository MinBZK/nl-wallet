import 'package:collection/collection.dart';
import 'package:wallet_core/core.dart';

import '../../../domain/model/card/wallet_card.dart';
import '../../../util/mapper/mapper.dart';
import '../../../wallet_core/typed/typed_wallet_core.dart';
import '../wallet_datasource.dart';

class WalletDataSourceImpl implements WalletDataSource {
  final TypedWalletCore _walletCore;
  final Mapper<AttestationPresentation, WalletCard> _cardMapper;

  WalletDataSourceImpl(this._walletCore, this._cardMapper);

  @override
  Future<List<WalletCard>> readAll() async {
    final cards = await _walletCore.observeCards().first.timeout(const Duration(seconds: 5));
    return _cardMapper.mapList(cards);
  }

  @override
  Future<WalletCard?> read(String attestationId) async {
    final cards = await readAll();
    return cards.firstWhereOrNull((element) => element.attestationId == attestationId);
  }

  @override
  Stream<List<WalletCard>> observeCards() => _walletCore.observeCards().map(_cardMapper.mapList);
}
