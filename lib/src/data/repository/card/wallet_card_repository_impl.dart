import '../../../domain/model/wallet_card.dart';
import '../../source/wallet_datasource.dart';
import 'wallet_card_repository.dart';

class WalletCardRepositoryImpl implements WalletCardRepository {
  final WalletDataSource dataSource;

  WalletCardRepositoryImpl(this.dataSource);

  @override
  Stream<List<WalletCard>> observeWalletCards() => dataSource.observeCards();

  @override
  Future<List<WalletCard>> readAll() async {
    return dataSource.readAll();
  }

  @override
  Future<void> create(WalletCard card) async {
    await dataSource.create(card);
  }

  @override
  Future<WalletCard> read(String cardId) async {
    return (await dataSource.read(cardId))!;
  }

  @override
  Future<void> delete(String cardId) async {
    await dataSource.delete(cardId);
  }

  @override
  Future<void> update(WalletCard card) async {
    await dataSource.update(card);
  }
}
