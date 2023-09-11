import 'package:collection/collection.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/wallet_card.dart';
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

  /// Finds single [AttributeKey] available in Wallet
  ///
  /// Returns `DataAttribute` when requested [AttributeKey] is found.
  /// Returns `null` when requested [AttributeKey] is not found.
  @override
  Future<DataAttribute?> find(AttributeKey key) async {
    final cards = await dataSource.readAll();
    for (WalletCard card in cards) {
      final result = card.attributes.firstWhereOrNull((attribute) => attribute.key == key);
      if (result != null) return result;
    }
    return null;
  }
}
