import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../../domain/model/timeline/timeline_attribute.dart';

abstract class TimelineAttributeRepository {
  /// Creates [TimelineAttribute] entry
  Future<void> create(TimelineAttribute attribute);

  /// Returns all wallet cards [TimelineAttribute]s sorted by date ASC (oldest first)
  Future<List<TimelineAttribute>> readAll();

  /// Returns all card specific [TimelineAttribute]s sorted by date ASC (oldest first)
  Future<List<TimelineAttribute>> readFiltered({required String cardId});

  /// Returns single [TimelineAttribute] by ID
  Future<TimelineAttribute> read({required String timelineAttributeId, String? cardId});

  /// Returns most recent [InteractionTimelineAttribute] for card filtered by [InteractionStatus]
  Future<InteractionTimelineAttribute?> readMostRecentInteraction(String cardId, InteractionStatus status);

  /// Returns most recent [OperationTimelineAttribute] for card filtered by [OperationStatus]
  Future<OperationTimelineAttribute?> readMostRecentOperation(String cardId, OperationStatus status);
}