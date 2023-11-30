import '../../../../data/repository/history/timeline_attribute_repository.dart';
import '../../../../util/extension/string_extension.dart';
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
      requestPurpose: requestPurpose.untranslated,
    );
    await timelineAttributeRepository.create(interaction);
  }
}
