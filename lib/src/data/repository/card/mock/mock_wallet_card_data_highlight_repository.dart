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
      case 'DIPLOMA_1':
        return _kMockDiplomaDataAttribute;
      case 'PASSPORT':
        return _kMockPassportDataAttribute;
      case 'DRIVING_LICENSE':
        return _kMockDrivingLicenseDataAttribute;
      case 'VOG':
        return _kMockVOGDataAttribute;
      default:
        throw UnimplementedError('No highlight configured for card: $cardId');
    }
  }
}
