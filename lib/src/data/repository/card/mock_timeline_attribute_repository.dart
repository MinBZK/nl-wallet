import 'package:collection/collection.dart';

import '../../../domain/model/timeline_attribute.dart';
import 'timeline_attribute_repository.dart';

class MockTimelineAttributeRepository implements TimelineAttributeRepository {
  MockTimelineAttributeRepository();

  @override
  Future<List<TimelineAttribute>> getAll(String cardId) async {
    switch (cardId) {
      case '1':
        return _kMockCardIdOneUsageAttributes;
      case '2':
        return _kMockCardIdTwoUsageAttributes;
      default:
        throw UnimplementedError();
    }
  }

  @override
  Future<InteractionAttribute?> getLastInteraction(String cardId, InteractionType type) async {
    switch (cardId) {
      case '1':
        return _getLastInteraction(_kMockCardIdOneUsageAttributes, type);
      case '2':
        return _getLastInteraction(_kMockCardIdTwoUsageAttributes, type);
      default:
        throw UnimplementedError();
    }
  }

  InteractionAttribute? _getLastInteraction(List<TimelineAttribute> attributes, InteractionType type) {
    return attributes.firstWhereOrNull((element) {
      return element is InteractionAttribute && element.interactionType == type;
    }) as InteractionAttribute?;
  }
}

final List<TimelineAttribute> _kMockCardIdOneUsageAttributes = [
  OperationAttribute(
    operationType: OperationType.extended,
    description: 'Deze kaart is geldig tot 12 oktober 2025',
    dateTime: DateTime.now().subtract(const Duration(minutes: 11)),
  ),
  InteractionAttribute(
    interactionType: InteractionType.success,
    organization: 'Organisatie X',
    dateTime: DateTime.now().subtract(const Duration(minutes: 57)),
  ),
  InteractionAttribute(
    interactionType: InteractionType.success,
    organization: 'Organisatie Y',
    dateTime: DateTime.now().subtract(const Duration(hours: 2)),
  ),
  InteractionAttribute(
    interactionType: InteractionType.rejected,
    organization: 'Organisatie Z',
    dateTime: DateTime.now().subtract(const Duration(hours: 13)),
  ),
  InteractionAttribute(
    interactionType: InteractionType.failed,
    organization: 'Organisatie A',
    dateTime: DateTime.now().subtract(const Duration(days: 2)),
  ),
  InteractionAttribute(
    interactionType: InteractionType.success,
    organization: 'Organisatie B',
    dateTime: DateTime.now().subtract(const Duration(days: 8)),
  ),
  InteractionAttribute(
    interactionType: InteractionType.rejected,
    organization: 'Organisatie C',
    dateTime: DateTime.now().subtract(const Duration(days: 35)),
  ),
];

final List<InteractionAttribute> _kMockCardIdTwoUsageAttributes = [
  InteractionAttribute(
    interactionType: InteractionType.rejected,
    organization: 'Organisatie K',
    dateTime: DateTime.now().subtract(const Duration(minutes: 2)),
  ),
  InteractionAttribute(
    interactionType: InteractionType.rejected,
    organization: 'Organisatie L',
    dateTime: DateTime.now().subtract(const Duration(hours: 3)),
  ),
];
