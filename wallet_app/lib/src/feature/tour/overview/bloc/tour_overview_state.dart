part of 'tour_overview_bloc.dart';

sealed class TourOverviewState extends Equatable {
  const TourOverviewState();
}

class TourInitial extends TourOverviewState {
  @override
  List<Object> get props => [];
}

class TourLoading extends TourOverviewState {
  @override
  List<Object> get props => [];
}

class TourLoaded extends TourOverviewState {
  final List<TourVideo> tourVideos;

  TourLoaded({required this.tourVideos}) : assert(tourVideos.isNotEmpty, 'At least one tour video should be provided');

  @override
  List<Object> get props => [tourVideos];
}

class TourLoadFailed extends TourOverviewState implements ErrorState {
  @override
  final ApplicationError error;

  const TourLoadFailed({required this.error});

  @override
  List<Object?> get props => [error];
}
