import 'package:collection/collection.dart';
import 'package:wallet_core/core.dart';

import '../../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../../../domain/model/timeline/timeline_attribute.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../history_repository.dart';
import '../timeline_attribute_repository.dart';

/// Temporarily implementing [TimelineAttributeRepository] for compatibility with mock setup,
/// as the [TimelineAttribute] is phased out, that implementation can be dropped.
class CoreHistoryRepository extends HistoryRepository implements TimelineAttributeRepository {
  final TypedWalletCore _walletCore;
  final Mapper<WalletEvent, TimelineAttribute> _walletEventMapper;

  CoreHistoryRepository(this._walletCore, this._walletEventMapper);

  @override
  Future<List<WalletEvent>> getHistory() async {
    return await _walletCore.getHistory();
  }

  @override
  Future<List<WalletEvent>> getHistoryForCard(String docType) async {
    return await _walletCore.getHistoryForCard(docType);
  }

  @override
  Future<List<TimelineAttribute>> readAll() async {
    final history = await getHistory();
    return _walletEventMapper.mapList(history);
  }

  @override
  Future<void> create(TimelineAttribute attribute) => throw UnimplementedError();

  @override
  Future<TimelineAttribute> read({required String timelineAttributeId, String? docType}) async {
    final all = await readAll();
    Iterable<TimelineAttribute> filteredAttributes;
    if (docType != null) {
      filteredAttributes = all.where((element) => element.attributesByDocType.containsKey(docType));
    } else {
      filteredAttributes = all;
    }
    return filteredAttributes.firstWhere((attribute) => attribute.id == timelineAttributeId);
  }

  @override
  Future<List<TimelineAttribute>> readFiltered({required String docType}) async {
    final all = await readAll();
    return all.where((element) => element.attributesByDocType.containsKey(docType)).toList();
  }

  @override
  Future<InteractionTimelineAttribute?> readMostRecentInteraction(String docType, InteractionStatus status) async {
    final cardHistory = _walletEventMapper.mapList(await getHistoryForCard(docType));
    cardHistory.sort((t1, t2) => t1.dateTime.compareTo(t2.dateTime));
    return cardHistory.reversed
        .whereType<InteractionTimelineAttribute>()
        .firstWhereOrNull((attribute) => attribute.status == status);
  }

  @override
  Future<OperationTimelineAttribute?> readMostRecentOperation(String docType, OperationStatus status) async {
    final cardHistory = _walletEventMapper.mapList(await getHistoryForCard(docType));
    cardHistory.sort((t1, t2) => t1.dateTime.compareTo(t2.dateTime));
    return cardHistory.reversed
        .whereType<OperationTimelineAttribute>()
        .firstWhereOrNull((attribute) => attribute.status == status);
  }
}
