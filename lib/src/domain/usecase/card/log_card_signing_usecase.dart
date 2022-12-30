import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../feature/verification/model/organization.dart';
import '../../model/attribute/data_attribute.dart';
import '../../model/document.dart';
import '../../model/policy/policy.dart';
import '../../model/timeline/signing_timeline_attribute.dart';

class LogCardSigningUseCase {
  final TimelineAttributeRepository timelineAttributeRepository;

  LogCardSigningUseCase(this.timelineAttributeRepository);

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
      dataAttributes: status == SigningStatus.success ? resolvedAttributes : [],
    );
    await timelineAttributeRepository.create(interaction);
  }
}
