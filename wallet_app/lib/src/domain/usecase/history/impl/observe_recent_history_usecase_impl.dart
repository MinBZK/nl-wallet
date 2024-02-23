import '../../../../data/repository/history/timeline_attribute_repository.dart';
import '../../../model/timeline/timeline_attribute.dart';
import '../observe_recent_history_usecase.dart';

class ObserveRecentHistoryUseCaseImpl implements ObserveRecentHistoryUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  ObserveRecentHistoryUseCaseImpl(this.timelineAttributeRepository);

  @override
  Stream<List<TimelineAttribute>> invoke() => timelineAttributeRepository.observeRecentHistory();
}
