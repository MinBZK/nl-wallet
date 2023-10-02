import '../../../../domain/model/wallet_card.dart';
import '../../../source/wallet_datasource.dart';
import '../wallet_card_repository.dart';

class WalletCardRepositoryImpl implements WalletCardRepository {
  final WalletDataSource _dataSource;

  WalletCardRepositoryImpl(this._dataSource);

  @override
  Stream<List<WalletCard>> observeWalletCards() => _dataSource.observeCards();

  @override
  Future<bool> exists(String cardId) async => await _dataSource.read(cardId) != null;

  @override
  Future<void> create(WalletCard card) async => await _dataSource.create(card);

  @override
  Future<List<WalletCard>> readAll() async => _dataSource.readAll();

  @override
  Future<WalletCard> read(String cardId) async => (await _dataSource.read(cardId))!;

  @override
  Future<void> update(WalletCard card) async => await _dataSource.update(card);

  @override
  Future<void> delete(String cardId) async => await _dataSource.delete(cardId);
}
