import 'package:collection/collection.dart';

import '../../../../domain/model/timeline/timeline_attribute.dart';
import '../../../source/wallet_datasource.dart';
import '../timeline_attribute_repository.dart';

class TimelineAttributeRepositoryImpl implements TimelineAttributeRepository {
  final WalletDataSource dataSource;

  TimelineAttributeRepositoryImpl(this.dataSource);

  @override
  Future<void> create(String cardId, TimelineAttribute attribute) async {
    dataSource.createTimelineAttribute(cardId, attribute);
  }

  @override
  Future<List<TimelineAttribute>> readAll() {
    return dataSource.readTimelineAttributes();
  }

  @override
  Future<List<TimelineAttribute>> readFiltered(String cardId) async {
    return dataSource.readTimelineAttributesByCardId(cardId);
  }

  @override
  Future<InteractionAttribute?> readLastInteraction(String cardId, InteractionStatus status) async {
    List<TimelineAttribute> attributes = await dataSource.readTimelineAttributesByCardId(cardId);
    attributes.sort((a, b) => b.dateTime.compareTo(a.dateTime)); // Sort by date/time DESC
    return _readLastInteraction(attributes, status);
  }

  @override
  Future<TimelineAttribute> read(String timelineAttributeId) {
    return dataSource.readTimelineAttributeById(timelineAttributeId);
  }

  InteractionAttribute? _readLastInteraction(List<TimelineAttribute> attributes, InteractionStatus status) {
    return attributes.firstWhereOrNull((element) {
      return element is InteractionAttribute && element.status == status;
    }) as InteractionAttribute?;
  }
}
