import '../../../domain/model/usage_attribute.dart';
import 'wallet_card_usage_attribute_repository.dart';

class MockWalletCardUsageAttributeRepository implements WalletCardUsageAttributeRepository {
  MockWalletCardUsageAttributeRepository();

  @override
  Future<UsageAttribute> getAll(String cardId) {
    throw UnimplementedError();
  }

  @override
  Future<UsageAttribute> getLatest(String cardId) async {
    switch (cardId) {
      case '1':
        return _kMockAirportUsageAttribute;
      case '2':
        return _kMockLotteryUsageAttribute;
      default:
        throw UnimplementedError();
    }
  }
}

final _kMockAirportUsageAttribute = UsageAttribute(
  value: 'Amsterdam Airport Schiphol',
  status: UsageStatus.success,
  dateTime: DateTime.now().subtract(const Duration(hours: 4)),
);

final _kMockLotteryUsageAttribute = UsageAttribute(
  value: 'Staatsloterij',
  status: UsageStatus.rejected,
  dateTime: DateTime.now().subtract(const Duration(days: 5)),
);
