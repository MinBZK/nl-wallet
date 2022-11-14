import '../../../domain/model/timeline_attribute.dart';

abstract class TimelineAttributeRepository {
  Future<List<TimelineAttribute>> getAll(String cardId);

  Future<InteractionAttribute?> getLastInteraction(String cardId, InteractionType type);
}
