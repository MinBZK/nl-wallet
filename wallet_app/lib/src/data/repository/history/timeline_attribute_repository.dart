import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../../domain/model/timeline/timeline_attribute.dart';

/// The [TimelineAttribute]s are entries that are rendered on the history screen within the app,
/// they were initially created for the mock builds but are now instantiated out of the [WalletEvent]s
/// which the core history api provides. For backwards compatibility the new [HistoryRepositoryImpl]
/// also implements this [TimelineAttributeRepository]. At some point we might want to phase out the
/// [TimelineAttribute]s completely in favor of a [WalletEvent] kind of model, to align more with what
/// the wallet_core provides.
abstract class TimelineAttributeRepository {
  /// Returns all wallet cards [TimelineAttribute]s sorted by date ASC (oldest first)
  Future<List<TimelineAttribute>> readAll();

  /// Returns all card specific [TimelineAttribute]s sorted by date ASC (oldest first)
  Future<List<TimelineAttribute>> readFiltered({required String docType});

  /// Returns most recent [InteractionTimelineAttribute] for card filtered by [InteractionStatus]
  Future<InteractionTimelineAttribute?> readMostRecentInteraction(String docType, InteractionStatus status);

  /// Returns most recent [OperationTimelineAttribute] for card filtered by [OperationStatus]
  Future<OperationTimelineAttribute?> readMostRecentOperation(String docType, OperationStatus status);
}
