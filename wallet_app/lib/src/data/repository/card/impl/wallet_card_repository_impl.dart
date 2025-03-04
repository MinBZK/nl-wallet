import '../../../../domain/model/card/wallet_card.dart';
import '../../../source/wallet_datasource.dart';
import '../wallet_card_repository.dart';

class WalletCardRepositoryImpl implements WalletCardRepository {
  final WalletDataSource _dataSource;

  WalletCardRepositoryImpl(this._dataSource);

  @override
  Stream<List<WalletCard>> observeWalletCards() => _dataSource.observeCards();

  @override
  Future<bool> exists(String docType) async => await _dataSource.read(docType) != null;

  @override
  Future<List<WalletCard>> readAll() async => _dataSource.readAll();

  @override
  Future<WalletCard> read(String docType) async => (await _dataSource.read(docType))!;
}
