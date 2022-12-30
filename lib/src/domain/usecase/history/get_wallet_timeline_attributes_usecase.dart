import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../wallet_constants.dart';
import '../../model/timeline/timeline_attribute.dart';

class GetWalletTimelineAttributesUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  GetWalletTimelineAttributesUseCase(this.timelineAttributeRepository);

  /// Returns all wallet cards [TimelineAttribute]s, sorted by date DESC (newest first)
  Future<List<TimelineAttribute>> invoke() async {
    await Future.delayed(kDefaultMockDelay);
    List<TimelineAttribute> results = await timelineAttributeRepository.readAll();
    results.sort((a, b) => b.dateTime.compareTo(a.dateTime)); // Sort by date/time DESC
    return results;
  }
}
