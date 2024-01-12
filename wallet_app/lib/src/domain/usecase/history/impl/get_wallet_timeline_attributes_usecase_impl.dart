import '../../../../data/repository/history/timeline_attribute_repository.dart';
import '../../../model/timeline/timeline_attribute.dart';
import '../get_wallet_timeline_attributes_usecase.dart';

class GetWalletTimelineAttributesUseCaseImpl implements GetWalletTimelineAttributesUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  GetWalletTimelineAttributesUseCaseImpl(this.timelineAttributeRepository);

  /// Returns all wallet cards [TimelineAttribute]s, sorted by date DESC (newest first)
  @override
  Future<List<TimelineAttribute>> invoke() async => await timelineAttributeRepository.readAll();
}
