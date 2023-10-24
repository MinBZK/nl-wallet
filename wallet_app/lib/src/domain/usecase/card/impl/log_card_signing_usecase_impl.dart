import '../../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../model/attribute/data_attribute.dart';
import '../../../model/document.dart';
import '../../../model/policy/policy.dart';
import '../../../model/timeline/signing_timeline_attribute.dart';
import '../log_card_signing_usecase.dart';

class LogCardSigningUseCaseImpl implements LogCardSigningUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  LogCardSigningUseCaseImpl(this.timelineAttributeRepository);

  @override
  Future<void> invoke(
    SigningStatus status,
    Policy policy,
    Organization organization,
    Document document,
    List<DataAttribute> resolvedAttributes,
  ) async {
    final interaction = SigningTimelineAttribute(
      status: status,
      policy: policy,
      document: document,
      dateTime: DateTime.now(),
      organization: organization,
      dataAttributes: resolvedAttributes,
    );
    await timelineAttributeRepository.create(interaction);
  }
}
