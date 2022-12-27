import '../../../../domain/model/data_highlight.dart';
import '../data_highlight_repository.dart';

part 'mock_data_highlight_repository.mocks.dart';

class MockDataHighlightRepository implements DataHighlightRepository {
  MockDataHighlightRepository();

  @override
  Future<DataHighlight> getLatest(String cardId) async {
    switch (cardId) {
      case 'PID_1':
        return _kMockPidDataAttribute;
      case 'DIPLOMA_1':
        return _kMockDiplomaDataAttribute;
      case 'DIPLOMA_2':
        return _kMockDiplomaDataAttribute;
      case 'DRIVING_LICENSE':
        return _kMockDrivingLicenseDataAttribute;
      case 'HEALTH_INSURANCE':
        return _kMockHealthInsuranceDataAttribute;
      case 'VOG':
        return _kMockVOGDataAttribute;
      case 'PASSPORT':
        return _kMockPassportDataAttribute;
      default:
        throw UnimplementedError('No highlight configured for card: $cardId');
    }
  }
}
