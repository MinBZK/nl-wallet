import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../wallet_constants.dart';
import '../../model/timeline_attribute.dart';

class GetTimelineAttributeUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  GetTimelineAttributeUseCase(this.timelineAttributeRepository);

  Future<TimelineAttribute> invoke(String attributeId) async {
    await Future.delayed(kDefaultMockDelay);
    return await timelineAttributeRepository.read(attributeId);
  }
}
