import '../../../domain/model/wallet_card_data_attribute.dart';
import 'wallet_card_data_repository.dart';

class MockWalletCardDataRepository implements WalletCardDataRepository {
  MockWalletCardDataRepository();

  @override
  Future<List<WalletCardDataAttribute>> getAll(String cardId) {
    throw UnimplementedError();
  }

  @override
  Future<WalletCardDataAttribute> getHighlight(String cardId) async {
    switch (cardId) {
      case '1':
        return _kMockPassportDataAttribute;
      case '2':
        return _kMockLicenseDataAttribute;
      default:
        throw UnimplementedError();
    }
  }
}

const _kMockPassportDataAttribute = WalletCardDataAttribute(
  content: 'Naam: W. de Bruijn\nGeldig t/m 23 januari 2024',
  image: 'assets/non-free/images/person_x.png',
);

const _kMockLicenseDataAttribute = WalletCardDataAttribute(
  content: 'Naam: W. de Bruijn\nGeldig t/m 23 januari 2024',
  image: 'assets/non-free/images/person_x.png',
);
