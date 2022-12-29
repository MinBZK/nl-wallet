import 'package:collection/collection.dart';

import '../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../feature/verification/model/organization.dart';
import '../../model/attribute/data_attribute.dart';
import '../../model/document.dart';
import '../../model/policy/policy.dart';
import '../../model/timeline/timeline_attribute.dart';

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
    final attributesByCardId = resolvedAttributes.groupListsBy((element) => element.sourceCardId);
    attributesByCardId.forEach((cardId, attributes) async {
      final cardInteraction = SigningAttribute(
        status: status,
        policy: policy,
        document: document,
        dateTime: DateTime.now(),
        organization: organization,
        attributes: _getFilteredAttributes(status, attributes),
        isSession: false,
      );
      await timelineAttributeRepository.create(cardId, cardInteraction);
    });

    // Session history
    final sessionInteraction = SigningAttribute(
      status: status,
      policy: policy,
      document: document,
      dateTime: DateTime.now(),
      organization: organization,
      attributes: _getFilteredAttributes(status, resolvedAttributes),
      isSession: true,
    );
    await timelineAttributeRepository.create(DateTime.now().microsecondsSinceEpoch.toString(), sessionInteraction);
  }

  /// Filters attributes for storage; only returns attributes for 'success' interaction
  List<DataAttribute> _getFilteredAttributes(SigningStatus status, List<DataAttribute> attributes) {
    if (status == SigningStatus.success) return attributes;
    return [];
  }
}
