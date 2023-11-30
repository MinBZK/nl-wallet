import '../../../../data/repository/history/timeline_attribute_repository.dart';
import '../../../../wallet_constants.dart';
import '../../../model/timeline/timeline_attribute.dart';
import '../get_wallet_card_timeline_attributes_usecase.dart';

class GetWalletCardTimelineAttributesUseCaseImpl implements GetWalletCardTimelineAttributesUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  GetWalletCardTimelineAttributesUseCaseImpl(this.timelineAttributeRepository);

  /// Returns all card specific [TimelineAttribute]s sorted by date DESC (newest first)
  @override
  Future<List<TimelineAttribute>> invoke(String docType) async {
    await Future.delayed(kDefaultMockDelay);
    List<TimelineAttribute> results = await timelineAttributeRepository.readFiltered(docType: docType);
    results.sort((a, b) => b.dateTime.compareTo(a.dateTime)); // Sort by date/time DESC
    return results;
  }
}
