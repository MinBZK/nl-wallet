import 'dart:collection';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/domain/model/event/wallet_events_page.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/feature/history/overview/bloc/history_overview_bloc.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

// Helpers to build state instances concisely in tests.
HistoryOverviewLoadSuccess _page(int pageNumber, List<WalletEvent> events, {bool hasNextPage = false}) =>
    HistoryOverviewLoadSuccess(
      pages: SplayTreeMap<int, List<WalletEvent>>()..[pageNumber] = List.of(events),
      lastLoadedPage: pageNumber,
      hasNextPage: hasNextPage,
    );

HistoryOverviewLoadSuccess _pages(Map<int, List<WalletEvent>> pages, {bool hasNextPage = false}) =>
    HistoryOverviewLoadSuccess(
      pages: SplayTreeMap<int, List<WalletEvent>>()..addAll(pages.map((k, v) => MapEntry(k, List.of(v)))),
      lastLoadedPage: pages.keys.reduce((a, b) => a > b ? a : b),
      hasNextPage: hasNextPage,
    );

void main() {
  late MockGetWalletEventsPageUseCase getWalletEventsPageUseCase;

  setUp(() {
    getWalletEventsPageUseCase = MockGetWalletEventsPageUseCase();
  });

  blocTest(
    'verify initial state',
    build: () => HistoryOverviewBloc(getWalletEventsPageUseCase),
    verify: (bloc) => expect(bloc.state, const HistoryOverviewInitial()),
  );

  blocTest(
    'verify transition to HistoryOverviewLoadFailure when events can not be loaded',
    build: () => HistoryOverviewBloc(getWalletEventsPageUseCase),
    setUp: () => when(
      getWalletEventsPageUseCase.invoke(
        page: anyNamed('page'),
        pageSize: anyNamed('pageSize'),
        currentPages: anyNamed('currentPages'),
      ),
    ).thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test'))),
    act: (bloc) => bloc.add(const HistoryOverviewLoadTriggered()),
    expect: () => [
      const HistoryOverviewLoadInProgress(),
      const HistoryOverviewLoadFailure(error: GenericError('', sourceError: 'test')),
    ],
  );

  blocTest(
    'verify transition to HistoryOverviewLoadSuccess when events can be loaded',
    build: () => HistoryOverviewBloc(getWalletEventsPageUseCase),
    setUp: () =>
        when(
          getWalletEventsPageUseCase.invoke(
            page: anyNamed('page'),
            pageSize: anyNamed('pageSize'),
            currentPages: anyNamed('currentPages'),
          ),
        ).thenAnswer(
          (_) async => Result.success(
            WalletEventsPage(
              pages: SplayTreeMap()..[0] = [WalletMockData.disclosureEvent],
              hasNextPage: false,
            ),
          ),
        ),
    act: (bloc) => bloc.add(const HistoryOverviewLoadTriggered()),
    expect: () => [
      const HistoryOverviewLoadInProgress(),
      _page(0, [WalletMockData.disclosureEvent]),
    ],
  );

  blocTest(
    'HistoryOverviewLoadNextPageTriggered appends events to the window',
    build: () => HistoryOverviewBloc(getWalletEventsPageUseCase),
    seed: () => _page(0, [WalletMockData.disclosureEvent], hasNextPage: true),
    setUp: () =>
        when(
          getWalletEventsPageUseCase.invoke(
            page: 1,
            pageSize: anyNamed('pageSize'),
            currentPages: anyNamed('currentPages'),
          ),
        ).thenAnswer(
          (_) async => Result.success(
            WalletEventsPage(
              pages: SplayTreeMap()
                ..[0] = [WalletMockData.disclosureEvent]
                ..[1] = [WalletMockData.issuanceEvent],
              hasNextPage: false,
            ),
          ),
        ),
    act: (bloc) => bloc.add(const HistoryOverviewLoadNextPageTriggered()),
    expect: () => [
      _page(0, [WalletMockData.disclosureEvent], hasNextPage: true).copyWith(isLoadingMore: true),
      _pages({
        0: [WalletMockData.disclosureEvent],
        1: [WalletMockData.issuanceEvent],
      }),
    ],
  );

  blocTest(
    'HistoryOverviewLoadNextPageTriggered is ignored when hasNextPage is false',
    build: () => HistoryOverviewBloc(getWalletEventsPageUseCase),
    seed: () => _page(0, [WalletMockData.disclosureEvent], hasNextPage: false),
    act: (bloc) => bloc.add(const HistoryOverviewLoadNextPageTriggered()),
    expect: () => [],
  );

  blocTest(
    'HistoryOverviewLoadNextPageTriggered is ignored when already loading more',
    build: () => HistoryOverviewBloc(getWalletEventsPageUseCase),
    seed: () => _page(0, [WalletMockData.disclosureEvent], hasNextPage: true).copyWith(isLoadingMore: true),
    act: (bloc) => bloc.add(const HistoryOverviewLoadNextPageTriggered()),
    expect: () => [],
  );

  blocTest(
    'next page load failure restores state without the loading indicator',
    build: () => HistoryOverviewBloc(getWalletEventsPageUseCase),
    seed: () => _page(0, [WalletMockData.disclosureEvent], hasNextPage: true),
    setUp: () => when(
      getWalletEventsPageUseCase.invoke(
        page: anyNamed('page'),
        pageSize: anyNamed('pageSize'),
        currentPages: anyNamed('currentPages'),
      ),
    ).thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test'))),
    act: (bloc) => bloc.add(const HistoryOverviewLoadNextPageTriggered()),
    expect: () => [
      _page(0, [WalletMockData.disclosureEvent], hasNextPage: true).copyWith(isLoadingMore: true),
      _page(0, [WalletMockData.disclosureEvent], hasNextPage: true).copyWith(isLoadingMore: false),
    ],
  );

  blocTest(
    'HistoryOverviewLoadTriggered requests the first page with an empty page window',
    build: () => HistoryOverviewBloc(getWalletEventsPageUseCase),
    setUp: () =>
        when(
          getWalletEventsPageUseCase.invoke(
            page: anyNamed('page'),
            pageSize: anyNamed('pageSize'),
            currentPages: anyNamed('currentPages'),
          ),
        ).thenAnswer(
          (_) async => Result.success(
            WalletEventsPage(pages: SplayTreeMap()..[0] = [WalletMockData.disclosureEvent], hasNextPage: false),
          ),
        ),
    act: (bloc) => bloc.add(const HistoryOverviewLoadTriggered()),
    verify: (_) {
      final captured = verify(
        getWalletEventsPageUseCase.invoke(
          page: captureAnyNamed('page'),
          pageSize: captureAnyNamed('pageSize'),
          currentPages: captureAnyNamed('currentPages'),
        ),
      ).captured;
      expect(captured, [0, 25, isEmpty]);
    },
  );

  blocTest(
    'HistoryOverviewLoadNextPageTriggered requests the next page using the current page window',
    build: () => HistoryOverviewBloc(getWalletEventsPageUseCase),
    seed: () => _page(0, [WalletMockData.disclosureEvent], hasNextPage: true),
    setUp: () =>
        when(
          getWalletEventsPageUseCase.invoke(
            page: anyNamed('page'),
            pageSize: anyNamed('pageSize'),
            currentPages: anyNamed('currentPages'),
          ),
        ).thenAnswer(
          (_) async => Result.success(
            WalletEventsPage(
              pages: SplayTreeMap()
                ..[0] = [WalletMockData.disclosureEvent]
                ..[1] = [WalletMockData.issuanceEvent],
              hasNextPage: false,
            ),
          ),
        ),
    act: (bloc) => bloc.add(const HistoryOverviewLoadNextPageTriggered()),
    verify: (_) {
      final captured = verify(
        getWalletEventsPageUseCase.invoke(
          page: captureAnyNamed('page'),
          pageSize: captureAnyNamed('pageSize'),
          currentPages: captureAnyNamed('currentPages'),
        ),
      ).captured;
      expect(captured[0], 1);
      expect(captured[1], 25);
      expect(captured[2], {
        0: [WalletMockData.disclosureEvent],
      });
    },
  );
}
