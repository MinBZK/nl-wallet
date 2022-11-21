import 'package:collection/collection.dart';

import '../../../../domain/model/timeline_attribute.dart';
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
  Future<List<TimelineAttribute>> readAll(String cardId) async {
    return dataSource.readTimelineAttributes(cardId);
  }

  @override
  Future<InteractionAttribute?> readLastInteraction(String cardId, InteractionType type) async {
    List<TimelineAttribute> attributes = await dataSource.readTimelineAttributes(cardId);
    return _readLastInteraction(attributes, type);
  }

  InteractionAttribute? _readLastInteraction(List<TimelineAttribute> attributes, InteractionType type) {
    return attributes.firstWhereOrNull((element) {
      return element is InteractionAttribute && element.interactionType == type;
    }) as InteractionAttribute?;
  }
}
