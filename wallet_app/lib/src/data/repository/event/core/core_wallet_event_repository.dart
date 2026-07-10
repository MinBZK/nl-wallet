import 'package:collection/collection.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/configuration/flutter_app_configuration.dart';
import '../../../../domain/model/event/wallet_event.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../util/mixin/pid_filter_mixin.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../wallet_event_repository.dart';

class CoreWalletEventRepository extends WalletEventRepository with PidFilterMixin {
  final TypedWalletCore _walletCore;
  final Mapper<core.WalletEvent, WalletEvent> _walletEventMapper;
  final Mapper<core.FlutterConfiguration, FlutterAppConfiguration> _flutterAppConfigurationMapper;

  CoreWalletEventRepository(this._walletCore, this._walletEventMapper, this._flutterAppConfigurationMapper);

  @override
  AppConfigurationProvider get configProvider =>
      () async => _flutterAppConfigurationMapper.map(await _walletCore.observeConfig().first);

  @override
  Future<List<WalletEvent>> getEvents({
    int? page,
    int? pageSize,
    bool removeDuplicatePidEvents = true,
  }) async {
    final isPaginated = page != null || pageSize != null;
    if (isPaginated && removeDuplicatePidEvents) {
      throw ArgumentError('Pagination cannot be combined with removeDuplicatePidEvents=true');
    }
    final coreEvents = await _walletCore.getHistory(page: page ?? 0, pageSize: pageSize ?? 0);
    final events = _walletEventMapper.mapList(coreEvents);
    if (!removeDuplicatePidEvents) return events;
    return filterDuplicatePidEvents(events);
  }

  @override
  Future<List<WalletEvent>> getEventsForCard(String attestationId) async {
    final coreEvents = await _walletCore.getHistoryForCard(attestationId);
    return _walletEventMapper.mapList(coreEvents);
  }

  @override
  Future<DisclosureEvent?> readMostRecentDisclosureEvent(String attestationId, EventStatus status) async {
    return _walletEventMapper
        .mapList(await _walletCore.getHistoryForCard(attestationId))
        .whereType<DisclosureEvent>()
        .firstWhereOrNull((e) => e.status == status);
  }

  @override
  Future<IssuanceEvent?> readMostRecentIssuanceEvent(String attestationId, EventStatus status) async {
    return _walletEventMapper
        .mapList(await _walletCore.getHistoryForCard(attestationId))
        .whereType<IssuanceEvent>()
        .firstWhereOrNull((e) => e.status == status);
  }

  @override
  Stream<List<WalletEvent>> observeRecentEvents({bool removeDuplicatePidEvents = true}) {
    final recentEventsStream = _walletCore.observeRecentHistory().map(_walletEventMapper.mapList);
    if (!removeDuplicatePidEvents) return recentEventsStream;
    return recentEventsStream.asyncMap(filterDuplicatePidEvents);
  }
}
