import '../../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../../wallet_constants.dart';
import '../../../model/timeline/timeline_attribute.dart';
import '../get_timeline_attribute_usecase.dart';

class GetTimelineAttributeUseCaseImpl implements GetTimelineAttributeUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  GetTimelineAttributeUseCaseImpl(this.timelineAttributeRepository);

  @override
  Future<TimelineAttribute> invoke({required String timelineAttributeId, required String? cardId}) async {
    await Future.delayed(kDefaultMockDelay);
    return await timelineAttributeRepository.read(timelineAttributeId: timelineAttributeId, cardId: cardId);
  }
}
