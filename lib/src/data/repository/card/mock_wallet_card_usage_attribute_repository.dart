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

const _kMockAirportUsageAttribute = UsageAttribute(
  value: '4 uur geleden gedeeld met Amsterdam Airport Schiphol',
  status: 'Gedeeld',
);

const _kMockLotteryUsageAttribute = UsageAttribute(
  value: '2 dagen geleden gedeeld Staatsloterij',
  status: 'Afgewezen',
);
