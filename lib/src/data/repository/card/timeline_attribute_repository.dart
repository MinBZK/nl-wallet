import '../../../domain/model/timeline_attribute.dart';

abstract class TimelineAttributeRepository {
  Future<void> create(String cardId, TimelineAttribute attribute);

  /// Returns all wallet cards [TimelineAttribute]s sorted by date ASC (oldest first)
  Future<List<TimelineAttribute>> readAll();

  /// Returns all card specific [TimelineAttribute]s sorted by date ASC (oldest first)
  Future<List<TimelineAttribute>> readFiltered(String cardId);

  Future<InteractionAttribute?> readLastInteraction(String cardId, InteractionType type);
}
