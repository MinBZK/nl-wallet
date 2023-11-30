import '../../../../data/repository/history/timeline_attribute_repository.dart';
import '../../../model/timeline/timeline_attribute.dart';
import '../get_timeline_attribute_usecase.dart';

class GetTimelineAttributeUseCaseImpl implements GetTimelineAttributeUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  GetTimelineAttributeUseCaseImpl(this.timelineAttributeRepository);

  @override
  Future<TimelineAttribute> invoke({required String timelineAttributeId, required String? docType}) async {
    return await timelineAttributeRepository.read(timelineAttributeId: timelineAttributeId, docType: docType);
  }
}
