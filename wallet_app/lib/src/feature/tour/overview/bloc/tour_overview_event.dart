part of 'tour_overview_bloc.dart';

abstract class TourOverviewEvent extends Equatable {
  const TourOverviewEvent();
}

class FetchVideosEvent extends TourOverviewEvent {
  const FetchVideosEvent();

  @override
  List<Object?> get props => [];
}
