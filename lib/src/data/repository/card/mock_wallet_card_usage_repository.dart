import '../../../domain/model/wallet_card_data_attribute.dart';
import 'wallet_card_usage_repository.dart';

class MockWalletCardUsageRepository implements WalletCardUsageRepository {
  MockWalletCardUsageRepository();

  @override
  Future<WalletCardDataAttribute> getAll(String cardId) {
    throw UnimplementedError();
  }

  @override
  Future<WalletCardDataAttribute> getHighlight(String cardId) async {
    switch (cardId) {
      case '1':
        return _kMockAirportDataAttribute;
      case '2':
        return _kMockLotteryDataAttribute;
      default:
        throw UnimplementedError();
    }
  }
}

const _kMockAirportDataAttribute = WalletCardDataAttribute(
  content: '4 uur geleden gedeeld met Amsterdam Airport Schiphol',
  image: null,
);

const _kMockLotteryDataAttribute = WalletCardDataAttribute(
  content: '2 dagen geleden gedeeld Staatsloterij',
  image: null,
);
