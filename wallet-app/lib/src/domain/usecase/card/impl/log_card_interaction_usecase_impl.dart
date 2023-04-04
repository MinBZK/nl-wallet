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
  Future<void> invoke(
    InteractionStatus status,
    Policy policy,
    Organization organization,
    List<DataAttribute> resolvedAttributes,
  ) async {
    final interaction = InteractionTimelineAttribute(
      status: status,
      policy: policy,
      dateTime: DateTime.now(),
      organization: organization,
      dataAttributes: resolvedAttributes,
    );
    await timelineAttributeRepository.create(interaction);
  }
}
