import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/model/tour/tour_video.dart';
import 'package:wallet/src/domain/usecase/tour/fetch_tour_videos_usecase.dart';
import 'package:wallet/src/feature/tour/overview/bloc/tour_overview_bloc.dart';
import 'package:wallet/src/util/extension/string_extension.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late FetchTourVideosUseCase fetchTourVideosUseCase;

  setUp(() {
    provideDummy<Result<List<TourVideo>>>(const Result.success([]));
  });

  blocTest(
    'when usecase returns success with 3 videos, emit TourLoaded with 3 videos',
    setUp: () {
      fetchTourVideosUseCase = MockFetchTourVideosUseCase();
      final sampleVideo = TourVideo(
        title: 'title'.untranslated,
        bulletPoints: 'bulletPoints'.untranslated,
        videoThumb: 'videoThumb'.untranslated,
        videoUrl: 'videoUrl'.untranslated,
        subtitleUrl: 'subtitleUrl'.untranslated,
      );
      when(fetchTourVideosUseCase.invoke()).thenAnswer(
        (_) async => Result.success([sampleVideo, sampleVideo, sampleVideo]),
      );
    },
    build: () => TourOverviewBloc(MockTourOverviewViewedUseCase(), fetchTourVideosUseCase),
    act: (bloc) => bloc.add(const FetchVideosEvent()),
    expect: () => [
      TourLoading(),
      isA<TourLoaded>().having(
        (it) => it.tourVideos,
        'Tour videos match expected count',
        hasLength(3),
      ),
    ],
  );

  blocTest(
    'when usecase throws, emit TourLoadFailed ',
    setUp: () {
      fetchTourVideosUseCase = MockFetchTourVideosUseCase();
      when(fetchTourVideosUseCase.invoke()).thenAnswer(
        (_) async => const Result.error(GenericError('test', sourceError: 'test')),
      );
    },
    build: () => TourOverviewBloc(MockTourOverviewViewedUseCase(), fetchTourVideosUseCase),
    act: (bloc) => bloc.add(const FetchVideosEvent()),
    expect: () => [
      TourLoading(),
      const TourLoadFailed(
        error: GenericError('test', sourceError: 'test'),
      ),
    ],
  );

  group('TourOverviewEvent', () {
    test('Verify equals of FetchVideosEvent works as expected', () {
      expect(const FetchVideosEvent(), equals(const FetchVideosEvent()));
    });

    test('Verify props is empty', () {
      expect(const FetchVideosEvent().props.isEmpty, isTrue);
    });
  });
}
