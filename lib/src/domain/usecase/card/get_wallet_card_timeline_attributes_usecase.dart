import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../wallet_constants.dart';
import '../../model/timeline_attribute.dart';

class GetWalletCardTimelineAttributesUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  GetWalletCardTimelineAttributesUseCase(this.timelineAttributeRepository);

  Future<List<TimelineAttribute>> invoke(String cardId) async {
    await Future.delayed(kDefaultMockDelay);
    return timelineAttributeRepository.readAll(cardId);
  }
}
