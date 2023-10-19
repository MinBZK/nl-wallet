import '../../model/attribute/data_attribute.dart';
import '../../model/organization.dart';
import '../../model/policy/policy.dart';
import '../../model/timeline/interaction_timeline_attribute.dart';

export '../../model/organization.dart';

abstract class LogCardInteractionUseCase {
  Future<void> invoke({
    required InteractionStatus status,
    required Policy policy,
    required Organization organization,
    required List<DataAttribute> resolvedAttributes,
    required String requestPurpose,
  });
}
