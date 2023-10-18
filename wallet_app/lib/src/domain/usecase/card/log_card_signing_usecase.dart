import '../../model/attribute/data_attribute.dart';
import '../../model/document.dart';
import '../../model/organization.dart';
import '../../model/policy/policy.dart';
import '../../model/timeline/signing_timeline_attribute.dart';

export '../../model/organization.dart';

abstract class LogCardSigningUseCase {
  Future<void> invoke(
    SigningStatus status,
    Policy policy,
    Organization organization,
    Document document,
    List<DataAttribute> resolvedAttributes,
  );
}
