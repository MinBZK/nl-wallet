import '../../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../../feature/verification/model/organization.dart';
import '../../../model/attribute/data_attribute.dart';
import '../../../model/policy/policy.dart';
import '../../../model/timeline/interaction_timeline_attribute.dart';
import '../log_card_interaction_usecase.dart';

class LogCardInteractionUseCaseImpl implements LogCardInteractionUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  LogCardInteractionUseCaseImpl(this.timelineAttributeRepository);

  @override
  Future<void> invoke({
    required InteractionStatus status,
    required Policy policy,
    required Organization organization,
    required List<DataAttribute> resolvedAttributes,
    required String requestPurpose,
  }) async {
    final interaction = InteractionTimelineAttribute(
      status: status,
      policy: policy,
      dateTime: DateTime.now(),
      organization: organization,
      dataAttributes: resolvedAttributes,
      requestPurpose: requestPurpose,
    );
    await timelineAttributeRepository.create(interaction);
  }
}
