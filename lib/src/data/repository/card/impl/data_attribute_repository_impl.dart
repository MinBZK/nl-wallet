import '../../../../domain/model/data_attribute.dart';
import '../../../source/wallet_datasource.dart';
import '../data_attribute_repository.dart';

class DataAttributeRepositoryImpl implements DataAttributeRepository {
  final WalletDataSource dataSource;

  DataAttributeRepositoryImpl(this.dataSource);

  @override
  Future<List<DataAttribute>?> getAll(String cardId) async {
    final walletCard = await dataSource.read(cardId);
    return walletCard?.attributes;
  }
}
