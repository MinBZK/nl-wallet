import 'package:wallet_core/core.dart';

import '../../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../../../domain/model/timeline/timeline_attribute.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../../card/timeline_attribute_repository.dart';
import '../history_repository.dart';

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
  Future<TimelineAttribute> read({required String timelineAttributeId, String? cardId}) => throw UnimplementedError();

  @override
  Future<List<TimelineAttribute>> readFiltered({required String cardId}) => throw UnimplementedError();

  @override
  Future<InteractionTimelineAttribute?> readMostRecentInteraction(String cardId, InteractionStatus status) =>
      throw UnimplementedError();

  @override
  Future<OperationTimelineAttribute?> readMostRecentOperation(String cardId, OperationStatus status) =>
      throw UnimplementedError();
}
