import 'package:collection/collection.dart';

import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../../data/repository/configuration/configuration_repository.dart';
import '../../../../data/repository/event/wallet_event_repository.dart';
import '../../../../util/mixin/pid_filter_mixin.dart';
import '../../../model/event/wallet_event.dart';
import '../../../model/result/result.dart';
import '../get_wallet_events_pid_usecase.dart';

class GetWalletEventsForPidUseCaseImpl extends GetWalletEventsForPidUseCase with PidFilterMixin {
  final ConfigurationRepository _configRepository;
  final WalletEventRepository _walletEventRepository;
  final WalletCardRepository _cardRepository;

  GetWalletEventsForPidUseCaseImpl(this._configRepository, this._walletEventRepository, this._cardRepository);

  @override
  AppConfigurationProvider get configProvider =>
      () => _configRepository.observeAppConfiguration.first;

  /// Returns all PID related [WalletEvent]s, sorted by date DESC (newest first). Duplicate CRUD events are filtered.
  @override
  Future<Result<List<WalletEvent>>> invoke() async {
    return tryCatch(
      () async {
        final config = await _configRepository.observeAppConfiguration.first;
        final allCards = await _cardRepository.readAll(filterDuplicatePids: false);
        final pidCards = allCards.where((card) => config.pidAttestationTypes.contains(card.attestationType));

        final pidEventFutures = pidCards.map((it) => _walletEventRepository.getEventsForCard(it.attestationId!));
        final mergedEvents = (await Future.wait(pidEventFutures)).flattened;
        final sortedEvents = mergedEvents.sortedBy((it) => it.dateTime).reversed;
        return filterDuplicatePidEvents(sortedEvents.toList());
      },
      'Failed to resolve events for PID',
    );
  }
}
