import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/tour/widget/tour_banner.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

const tourBannerSize = Size(390, 64);

void main() {
  group('goldens', () {
    testGoldens(
      'light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const TourBanner(),
          surfaceSize: tourBannerSize,
        );
        await screenMatchesGolden('light');
      },
    );

    testGoldens(
      'light focused',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const TourBanner(),
          surfaceSize: tourBannerSize,
        );
        await tester.sendKeyEvent(LogicalKeyboardKey.tab); // Trigger focused state
        await tester.pumpAndSettle();
        await screenMatchesGolden('focused.light');
      },
    );

    testGoldens(
      'light scaled',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const TourBanner(),
          surfaceSize: const Size(390, 128),
          textScaleSize: 2,
        );
        await screenMatchesGolden('scaled.light');
      },
    );

    testGoldens(
      'dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const TourBanner(),
          brightness: Brightness.dark,
          surfaceSize: tourBannerSize,
        );
        await screenMatchesGolden('dark');
      },
    );

    testGoldens(
      'dark focused',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const TourBanner(),
          brightness: Brightness.dark,
          surfaceSize: tourBannerSize,
        );
        await tester.sendKeyEvent(LogicalKeyboardKey.tab); // Trigger focused state
        await tester.pumpAndSettle();
        await screenMatchesGolden('focused.dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('banner shows title', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TourBanner(),
      );
      final l10n = await TestUtils.englishLocalizations;
      final widgetFinder = find.text(l10n.tourBannerTitle);
      expect(widgetFinder, findsOneWidget);
    });
  });
}
