import '../../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../model/timeline/interaction_timeline_attribute.dart';
import '../has_previously_interacted_with_organization_usecase.dart';

class HasPreviouslyInteractedWithOrganizationUseCaseImpl implements HasPreviouslyInteractedWithOrganizationUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  HasPreviouslyInteractedWithOrganizationUseCaseImpl(this.timelineAttributeRepository);

  @override
  Future<bool> invoke(String organizationId) async {
    final entries = await timelineAttributeRepository.readAll();
    return entries
        .whereType<InteractionTimelineAttribute>()
        .where((element) => element.status == InteractionStatus.success)
        .any((element) => element.organization.id == organizationId);
  }
}
