import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../wallet_constants.dart';
import '../../model/timeline/timeline_attribute.dart';

class GetWalletCardTimelineAttributesUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  GetWalletCardTimelineAttributesUseCase(this.timelineAttributeRepository);

  /// Returns all card specific [TimelineAttribute]s sorted by date DESC (newest first)
  Future<List<TimelineAttribute>> invoke(String cardId) async {
    await Future.delayed(kDefaultMockDelay);
    List<TimelineAttribute> results = await timelineAttributeRepository.readFiltered(cardId);
    results.sort((a, b) => b.dateTime.compareTo(a.dateTime)); // Sort by date/time DESC
    return results;
  }
}
