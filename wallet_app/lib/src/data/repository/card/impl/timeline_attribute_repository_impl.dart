import 'package:collection/collection.dart';

import '../../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../../../domain/model/timeline/timeline_attribute.dart';
import '../../../source/wallet_datasource.dart';
import '../timeline_attribute_repository.dart';

class TimelineAttributeRepositoryImpl implements TimelineAttributeRepository {
  final WalletDataSource _dataSource;

  TimelineAttributeRepositoryImpl(this._dataSource);

  @override
  Future<void> create(TimelineAttribute attribute) async {
    _dataSource.createTimelineAttribute(attribute);
  }

  @override
  Future<List<TimelineAttribute>> readAll() {
    return _dataSource.readTimelineAttributes();
  }

  @override
  Future<List<TimelineAttribute>> readFiltered({required String cardId}) async {
    return _dataSource.readTimelineAttributesByCardId(cardId: cardId);
  }

  @override
  Future<TimelineAttribute> read({required String timelineAttributeId, String? cardId}) {
    return _dataSource.readTimelineAttributeById(timelineAttributeId: timelineAttributeId, cardId: cardId);
  }

  @override
  Future<InteractionTimelineAttribute?> readMostRecentInteraction(String cardId, InteractionStatus status) async {
    List<TimelineAttribute> attributes = await _dataSource.readTimelineAttributesByCardId(cardId: cardId);
    return _readMostRecentInteraction(attributes, status);
  }

  @override
  Future<OperationTimelineAttribute?> readMostRecentOperation(String cardId, OperationStatus status) async {
    List<TimelineAttribute> attributes = await _dataSource.readTimelineAttributesByCardId(cardId: cardId);
    return _readMostRecentOperation(attributes, status);
  }

  InteractionTimelineAttribute? _readMostRecentInteraction(
    List<TimelineAttribute> attributes,
    InteractionStatus status,
  ) {
    // Copy list & sort by date/time DESC
    List<TimelineAttribute> copy = List.from(attributes);
    copy.sort((a, b) => b.dateTime.compareTo(a.dateTime));

    // Return first element that matches the status
    return copy.whereType<InteractionTimelineAttribute>().firstWhereOrNull((element) {
      return element.status == status;
    });
  }

  OperationTimelineAttribute? _readMostRecentOperation(
    List<TimelineAttribute> attributes,
    OperationStatus status,
  ) {
    // Copy list & sort by date/time DESC
    List<TimelineAttribute> copy = List.from(attributes);
    copy.sort((a, b) => b.dateTime.compareTo(a.dateTime)); // Sort by date/time DESC

    // Return first element that matches the status
    return copy.whereType<OperationTimelineAttribute>().firstWhereOrNull((element) {
      return element.status == status;
    });
  }
}
