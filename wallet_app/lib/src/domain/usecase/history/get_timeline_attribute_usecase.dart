import '../../model/timeline/timeline_attribute.dart';

abstract class GetTimelineAttributeUseCase {
  Future<TimelineAttribute> invoke({required String timelineAttributeId, required String? docType});
}
