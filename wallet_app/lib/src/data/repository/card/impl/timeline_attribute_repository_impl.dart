import 'package:collection/collection.dart';

import '../../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../../domain/model/timeline/timeline_attribute.dart';
import '../../../source/wallet_datasource.dart';
import '../timeline_attribute_repository.dart';

class TimelineAttributeRepositoryImpl implements TimelineAttributeRepository {
  final WalletDataSource dataSource;

  TimelineAttributeRepositoryImpl(this.dataSource);

  @override
  Future<void> create(TimelineAttribute attribute) async {
    dataSource.createTimelineAttribute(attribute);
  }

  @override
  Future<List<TimelineAttribute>> readAll() {
    return dataSource.readTimelineAttributes();
  }

  @override
  Future<List<TimelineAttribute>> readFiltered({required String cardId}) async {
    return dataSource.readTimelineAttributesByCardId(cardId: cardId);
  }

  @override
  Future<InteractionTimelineAttribute?> readLastInteraction(String cardId, InteractionStatus status) async {
    List<TimelineAttribute> attributes = await dataSource.readTimelineAttributesByCardId(cardId: cardId);
    attributes.sort((a, b) => b.dateTime.compareTo(a.dateTime)); // Sort by date/time DESC
    return _readLastInteraction(attributes, status);
  }

  @override
  Future<TimelineAttribute> read({required String timelineAttributeId, String? cardId}) {
    return dataSource.readTimelineAttributeById(timelineAttributeId: timelineAttributeId, cardId: cardId);
  }

  InteractionTimelineAttribute? _readLastInteraction(List<TimelineAttribute> attributes, InteractionStatus status) {
    return attributes.firstWhereOrNull((element) {
      return element is InteractionTimelineAttribute && element.status == status;
    }) as InteractionTimelineAttribute?;
  }
}
