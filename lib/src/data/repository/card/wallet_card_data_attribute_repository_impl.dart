import '../../../domain/model/data_attribute.dart';
import '../../source/wallet_datasource.dart';
import 'wallet_card_data_attribute_repository.dart';

class WalletCardDataAttributeRepositoryImpl implements WalletCardDataAttributeRepository {
  final WalletDataSource dataSource;

  WalletCardDataAttributeRepositoryImpl(this.dataSource);

  @override
  Future<List<DataAttribute>?> getAll(String cardId) async {
    final walletCard = await dataSource.read(cardId);
    return walletCard?.attributes;
  }
}
