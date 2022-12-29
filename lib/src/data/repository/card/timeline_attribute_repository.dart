import '../../../domain/model/timeline_attribute.dart';

abstract class TimelineAttributeRepository {
  /// Creates [TimelineAttribute] entry
  Future<void> create(String cardId, TimelineAttribute attribute);

  /// Returns all wallet cards [TimelineAttribute]s sorted by date ASC (oldest first)
  Future<List<TimelineAttribute>> readAll();

  /// Returns all card specific [TimelineAttribute]s sorted by date ASC (oldest first)
  Future<List<TimelineAttribute>> readFiltered(String cardId);

  /// Returns single [TimelineAttribute] by ID
  Future<TimelineAttribute> read(String timelineAttributeId);

  Future<InteractionAttribute?> readLastInteraction(String cardId, InteractionType type);
}
