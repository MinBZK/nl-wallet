import 'dart:collection';

import 'package:collection/collection.dart';

import '../../../../data/repository/configuration/configuration_repository.dart';
import '../../../../data/repository/event/wallet_event_repository.dart';
import '../../../../util/mixin/pid_filter_mixin.dart';
import '../../../model/event/wallet_event.dart';
import '../../../model/event/wallet_events_page.dart';
import '../../../model/result/result.dart';
import '../get_wallet_events_page_usecase.dart';

class GetWalletEventsPageUseCaseImpl extends GetWalletEventsPageUseCase with PidFilterMixin {
  final ConfigurationRepository _configRepository;
  final WalletEventRepository _walletEventRepository;

  GetWalletEventsPageUseCaseImpl(this._configRepository, this._walletEventRepository);

  @override
  AppConfigurationProvider get configProvider =>
      () => _configRepository.observeAppConfiguration.first;

  @override
  Future<Result<WalletEventsPage>> invoke({
    required int page,
    required int pageSize,
    required SplayTreeMap<int, List<WalletEvent>> currentPages,
  }) async {
    return tryCatch(
      () async {
        final newEvents = await _walletEventRepository.getEvents(
          page: page,
          pageSize: pageSize,
          removeDuplicatePidEvents: false,
        );
        final hasNextPage = newEvents.length >= pageSize;
        final updatedPages = SplayTreeMap<int, List<WalletEvent>>.from(currentPages);

        final previousPage = currentPages.keys.lastOrNull;
        if (previousPage == null) {
          updatedPages[page] = await filterDuplicatePidEvents(newEvents);
          return WalletEventsPage(pages: updatedPages, hasNextPage: hasNextPage);
        }

        // Deduplicate events across the boundary of the previous and next page.
        // This works because the events are sorted and thus duplications are (extremely)
        // unlikely to occur across >2 pages as that would require a single event to
        // occur (pageSize * 2) times.
        final previousPageEvents = currentPages[previousPage]!;
        final filteredEvents = await filterDuplicatePidEvents(previousPageEvents + newEvents);
        updatedPages[previousPage] = previousPageEvents.where(filteredEvents.contains).toList();
        updatedPages[page] = newEvents.where(filteredEvents.contains).toList();

        return WalletEventsPage(pages: updatedPages, hasNextPage: hasNextPage);
      },
      'Failed to get wallet events page',
    );
  }
}
