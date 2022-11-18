import 'package:collection/collection.dart';

import '../../../domain/model/timeline_attribute.dart';
import 'timeline_attribute_repository.dart';

part 'mock_timeline_attribute_repository.mocks.dart';

class MockTimelineAttributeRepository implements TimelineAttributeRepository {
  MockTimelineAttributeRepository();

  @override
  Future<List<TimelineAttribute>> getAll(String cardId) async {
    switch (cardId) {
      case 'PID_1':
        return _kMockCardIdPidOneUsageAttributes;
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
      case 'PID_1':
        return _getLastInteraction(_kMockCardIdPidOneUsageAttributes, type);
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
