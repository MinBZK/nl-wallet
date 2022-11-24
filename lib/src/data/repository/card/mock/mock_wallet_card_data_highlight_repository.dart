import '../../../../domain/model/data_highlight.dart';
import '../wallet_card_data_highlight_repository.dart';

part 'mock_wallet_card_data_highlight_repository.mocks.dart';

class MockWalletCardDataHighlightRepository implements WalletCardDataHighlightRepository {
  MockWalletCardDataHighlightRepository();

  @override
  Future<DataHighlight> getLatest(String cardId) async {
    switch (cardId) {
      case 'PID_1':
        return _kMockPidDataAttribute;
      case '1':
        return _kMockPassportDataAttribute;
      case '2':
        return _kMockDrivingLicenseDataAttribute;
      default:
        throw UnimplementedError();
    }
  }
}
