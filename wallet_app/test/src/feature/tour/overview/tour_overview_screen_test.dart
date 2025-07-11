import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/tour/tour_video.dart';
import 'package:wallet/src/feature/tour/overview/bloc/tour_overview_bloc.dart';
import 'package:wallet/src/feature/tour/overview/tour_overview_screen.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/wallet_assets.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

class MockTourBloc extends MockBloc<TourOverviewEvent, TourOverviewState> implements TourOverviewBloc {}

void main() {
  Future<List<TourVideo>> generateSampleVideoList() async {
    final l10n = await TestUtils.englishLocalizations;
    return [
      TourVideo(
        title: l10n.videoTitle_intro.untranslated,
        bulletPoints: l10n.videoBulletPoints_intro.untranslated,
        videoThumb: 'assets/non-free/images/tour_video_thumb_${WalletAssets.video_slugs[0]}_en.png'.untranslated,
        videoUrl: 'videoUrl'.untranslated,
        subtitleUrl: 'subtitleUrl'.untranslated,
      ),
      TourVideo(
        title: l10n.videoTitle_cards_insight.untranslated,
        bulletPoints: l10n.videoBulletPoints_cards_insight.untranslated,
        videoThumb: 'assets/non-free/images/tour_video_thumb_${WalletAssets.video_slugs[1]}_en.png'.untranslated,
        videoUrl: 'videoUrl'.untranslated,
        subtitleUrl: 'subtitleUrl'.untranslated,
      ),
    ];
  }

  group('goldens', () {
    testGoldens('light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TourOverviewScreen().withState<TourOverviewBloc, TourOverviewState>(
          MockTourBloc(),
          TourLoaded(tourVideos: await generateSampleVideoList()),
        ),
      );
      await screenMatchesGolden('light');
    });

    testGoldens('dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TourOverviewScreen().withState<TourOverviewBloc, TourOverviewState>(
          MockTourBloc(),
          TourLoaded(tourVideos: await generateSampleVideoList()),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('dark');
    });

    testGoldens('light scaled', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TourOverviewScreen().withState<TourOverviewBloc, TourOverviewState>(
          MockTourBloc(),
          TourLoaded(tourVideos: await generateSampleVideoList()),
        ),
        textScaleSize: 2,
      );
      await screenMatchesGolden('scaled.light');
    });

    testGoldens('light landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TourOverviewScreen().withState<TourOverviewBloc, TourOverviewState>(
          MockTourBloc(),
          TourLoaded(tourVideos: await generateSampleVideoList()),
        ),
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('landscape.light');
    });

    testGoldens('light - loading', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TourOverviewScreen().withState<TourOverviewBloc, TourOverviewState>(
          MockTourBloc(),
          TourLoading(),
        ),
      );
      await screenMatchesGolden('light.loading');
    });

    testGoldens('light - error', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TourOverviewScreen().withState<TourOverviewBloc, TourOverviewState>(
          MockTourBloc(),
          const TourLoadFailed(error: GenericError('test', sourceError: 'test')),
        ),
      );
      await screenMatchesGolden('light.error');
    });
  });

  group('widgets', () {
    testWidgets('expected tour videos are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TourOverviewScreen().withState<TourOverviewBloc, TourOverviewState>(
          MockTourBloc(),
          TourLoaded(tourVideos: await generateSampleVideoList()),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      final videoTitles = [
        l10n.videoTitle_intro,
        l10n.videoTitle_cards_insight,
      ];

      // Check if all video titles are present
      for (final title in videoTitles) {
        final titleFinder = find.text(title);

        // Scroll if needed
        await tester.ensureVisible(titleFinder);
        await tester.pumpAndSettle();

        // Verify the title is found
        expect(titleFinder, findsOneWidget);
      }
    });
  });
}
