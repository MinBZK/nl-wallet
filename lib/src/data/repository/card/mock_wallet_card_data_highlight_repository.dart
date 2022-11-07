import '../../../domain/model/data_highlight.dart';
import 'wallet_card_data_highlight_repository.dart';

class MockWalletCardDataHighlightRepository implements WalletCardDataHighlightRepository {
  MockWalletCardDataHighlightRepository();

  @override
  Future<DataHighlight> getLatest(String cardId) async {
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

const _kMockPassportDataAttribute = DataHighlight(
  title: 'Naam: W. de Bruijn',
  subtitle: 'Geldig t/m 23 januari 2024',
  image: 'assets/non-free/images/person_x.png',
);

const _kMockLicenseDataAttribute = DataHighlight(
  title: 'Naam: W. de Bruijn',
  subtitle: 'Geldig t/m 23 januari 2024',
  image: 'assets/non-free/images/person_x.png',
);
