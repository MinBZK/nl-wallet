import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/history/overview/bloc/history_overview_bloc.dart';
import 'package:wallet/src/feature/tour/overview/tour_overview_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

class MockHistoryOverviewBloc extends MockBloc<HistoryOverviewEvent, HistoryOverviewState>
    implements HistoryOverviewBloc {}

void main() {
  group('goldens', () {
    testGoldens('light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TourOverviewScreen(),
      );
      await screenMatchesGolden('light');
    });

    testGoldens('dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TourOverviewScreen(),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('dark');
    });

    testGoldens('light scaled', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TourOverviewScreen(),
        textScaleSize: 2,
      );
      await screenMatchesGolden('scaled.light');
    });

    testGoldens('light landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TourOverviewScreen(),
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('landscape.light');
    });
  });

  group('widgets', () {
    testWidgets('expected tour videos are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TourOverviewScreen(),
      );

      final l10n = await TestUtils.englishLocalizations;
      final videoTitles = [
        l10n.tourOverviewVideo1Title,
        l10n.tourOverviewVideo2Title,
        l10n.tourOverviewVideo3Title,
        l10n.tourOverviewVideo4Title,
        l10n.tourOverviewVideo5Title,
        l10n.tourOverviewVideo6Title,
        l10n.tourOverviewVideo7Title,
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
