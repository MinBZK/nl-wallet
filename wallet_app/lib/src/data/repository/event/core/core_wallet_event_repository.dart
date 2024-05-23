import 'package:collection/collection.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/event/wallet_event.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../wallet_event_repository.dart';

class CoreWalletEventRepository extends WalletEventRepository {
  final TypedWalletCore _walletCore;
  final Mapper<core.WalletEvent, WalletEvent> _walletEventMapper;

  CoreWalletEventRepository(this._walletCore, this._walletEventMapper);

  @override
  Future<List<WalletEvent>> getEvents() async {
    final coreEvents = await _walletCore.getHistory();
    return _walletEventMapper.mapList(coreEvents);
  }

  @override
  Future<List<WalletEvent>> getEventsForCard(String docType) async {
    final coreEvents = await _walletCore.getHistoryForCard(docType);
    return _walletEventMapper.mapList(coreEvents);
  }

  @override
  Future<DisclosureEvent?> readMostRecentDisclosureEvent(String docType, EventStatus status) async {
    return _walletEventMapper
        .mapList(await _walletCore.getHistoryForCard(docType))
        .whereType<DisclosureEvent>()
        .firstWhereOrNull((e) => e.status == status);
  }

  @override
  Future<IssuanceEvent?> readMostRecentIssuanceEvent(String docType, EventStatus status) async {
    return _walletEventMapper
        .mapList(await _walletCore.getHistoryForCard(docType))
        .whereType<IssuanceEvent>()
        .firstWhereOrNull((e) => e.status == status);
  }

  @override
  Stream<List<WalletEvent>> observeRecentEvents() =>
      _walletCore.observeRecentHistory().map((event) => _walletEventMapper.mapList(event));
}
