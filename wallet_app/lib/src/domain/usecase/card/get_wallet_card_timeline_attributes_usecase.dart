import '../../model/timeline/timeline_attribute.dart';

abstract class GetWalletCardTimelineAttributesUseCase {
  Future<List<TimelineAttribute>> invoke(String docType);
}
