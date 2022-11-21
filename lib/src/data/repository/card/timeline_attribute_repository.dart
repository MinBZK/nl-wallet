import '../../../domain/model/timeline_attribute.dart';

abstract class TimelineAttributeRepository {
  Future<void> create(String cardId, TimelineAttribute attribute);

  Future<List<TimelineAttribute>> readAll(String cardId);

  Future<InteractionAttribute?> readLastInteraction(String cardId, InteractionType type);
}
