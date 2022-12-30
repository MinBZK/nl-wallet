import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../feature/verification/model/organization.dart';
import '../../model/attribute/data_attribute.dart';
import '../../model/policy/policy.dart';
import '../../model/timeline/interaction_timeline_attribute.dart';

class LogCardInteractionUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  LogCardInteractionUseCase(this.timelineAttributeRepository);

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
      dataAttributes: status == InteractionStatus.success ? resolvedAttributes : [],
    );
    await timelineAttributeRepository.create(interaction);
  }
}
