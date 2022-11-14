import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../wallet_constants.dart';
import '../../model/timeline_attribute.dart';

class GetWalletCardTimelineUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  GetWalletCardTimelineUseCase(this.timelineAttributeRepository);

  Future<List<TimelineAttribute>> getAll(String cardId) async {
    await Future.delayed(kDefaultMockDelay);
    return timelineAttributeRepository.getAll(cardId);
  }
}
