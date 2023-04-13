import '../../model/timeline/timeline_attribute.dart';

abstract class GetWalletTimelineAttributesUseCase {
  Future<List<TimelineAttribute>> invoke();
}
