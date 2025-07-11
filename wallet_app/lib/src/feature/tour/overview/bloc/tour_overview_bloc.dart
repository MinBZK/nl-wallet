import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/bloc/error_state.dart';
import '../../../../domain/model/result/application_error.dart';
import '../../../../domain/model/tour/tour_video.dart';
import '../../../../domain/usecase/tour/fetch_tour_videos_usecase.dart';
import '../../../../domain/usecase/tour/tour_overview_viewed_usecase.dart';

part 'tour_overview_event.dart';
part 'tour_overview_state.dart';

class TourOverviewBloc extends Bloc<TourOverviewEvent, TourOverviewState> {
  final TourOverviewViewedUseCase _tourOverviewViewedUseCase;
  final FetchTourVideosUseCase _fetchTourVideosUseCase;

  TourOverviewBloc(
    this._tourOverviewViewedUseCase,
    this._fetchTourVideosUseCase,
  ) : super(TourInitial()) {
    on<FetchVideosEvent>(_fetchTourVideos);
  }

  Future<void> _fetchTourVideos(FetchVideosEvent event, Emitter<TourOverviewState> emit) async {
    emit(TourLoading());
    final result = await _fetchTourVideosUseCase.invoke();
    await result.process(
      onSuccess: (videos) {
        emit(TourLoaded(tourVideos: videos));
        unawaited(_tourOverviewViewedUseCase.invoke());
      },
      onError: (error) {
        Fimber.e('Failed to fetch videos', ex: error);
        emit(TourLoadFailed(error: error));
      },
    );
  }
}
