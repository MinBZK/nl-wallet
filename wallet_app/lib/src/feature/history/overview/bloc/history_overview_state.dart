part of 'history_overview_bloc.dart';

sealed class HistoryOverviewState extends Equatable {
  const HistoryOverviewState();

  @override
  List<Object?> get props => [];
}

class HistoryOverviewInitial extends HistoryOverviewState {
  const HistoryOverviewInitial();
}

class HistoryOverviewLoadInProgress extends HistoryOverviewState {
  const HistoryOverviewLoadInProgress();
}

class HistoryOverviewLoadSuccess extends HistoryOverviewState {
  /// Pages currently held in memory, keyed by page number.
  final SplayTreeMap<int, List<WalletEvent>> pages;
  final int lastLoadedPage;
  final bool hasNextPage;
  final bool isLoadingMore;

  /// Events grouped by year-month, preserving newest-first order within each section.
  final Map<DateTime, List<WalletEvent>> eventsByYearMonth;

  HistoryOverviewLoadSuccess({
    required SplayTreeMap<int, List<WalletEvent>> pages,
    required int lastLoadedPage,
    required bool hasNextPage,
    bool isLoadingMore = false,
  }) : this._(
         pages: pages,
         lastLoadedPage: lastLoadedPage,
         hasNextPage: hasNextPage,
         isLoadingMore: isLoadingMore,
       );

  HistoryOverviewLoadSuccess._({
    required this.pages,
    required this.lastLoadedPage,
    required this.hasNextPage,
    required this.isLoadingMore,
  }) : eventsByYearMonth = pages.values.flattened.groupListsBy((event) => event.dateTime.yearMonth);

  HistoryOverviewLoadSuccess copyWith({bool? isLoadingMore}) => HistoryOverviewLoadSuccess(
    pages: pages,
    lastLoadedPage: lastLoadedPage,
    hasNextPage: hasNextPage,
    isLoadingMore: isLoadingMore ?? this.isLoadingMore,
  );

  @override
  List<Object?> get props => [
    pages,
    lastLoadedPage,
    hasNextPage,
    isLoadingMore,
    eventsByYearMonth,
  ];
}

class HistoryOverviewLoadFailure extends HistoryOverviewState implements ErrorState {
  @override
  final ApplicationError error;

  const HistoryOverviewLoadFailure({required this.error});

  @override
  List<Object?> get props => [error];
}
