import '../../model/timeline/timeline_attribute.dart';

abstract class ObserveRecentHistoryUseCase {
  Stream<List<TimelineAttribute>> invoke();
}
