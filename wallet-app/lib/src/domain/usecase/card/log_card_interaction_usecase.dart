import '../../../feature/verification/model/organization.dart';
import '../../model/attribute/data_attribute.dart';
import '../../model/policy/policy.dart';
import '../../model/timeline/interaction_timeline_attribute.dart';

abstract class LogCardInteractionUseCase {
  Future<void> invoke(
    InteractionStatus status,
    Policy policy,
    Organization organization,
    List<DataAttribute> resolvedAttributes,
  );
}
