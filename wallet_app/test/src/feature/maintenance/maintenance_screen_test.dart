import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/configuration/maintenance_window.dart';
import 'package:wallet/src/feature/common/widget/button/icon/help_icon_button.dart';
import 'package:wallet/src/feature/common/widget/text/body_text.dart';
import 'package:wallet/src/feature/maintenance/maintenance_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('MaintenanceScreen Light - Single day', (tester) async {
      await withClock(Clock.fixed(DateTime(2025, 1, 15, 10, 30)), () async {
        final now = clock.now();
        final maintenance = MaintenanceWindow(
          startDateTime: now,
          endDateTime: now.add(const Duration(hours: 2)),
        );

        await tester.pumpWidgetWithAppWrapper(
          MaintenanceScreen(maintenanceWindow: maintenance),
        );
        await screenMatchesGolden('single_day.light');
      });
    });

    testGoldens('MaintenanceScreen Dark - Single day', (tester) async {
      await withClock(Clock.fixed(DateTime(2025, 1, 15, 10, 30)), () async {
        final now = clock.now();
        final maintenance = MaintenanceWindow(
          startDateTime: now,
          endDateTime: now.add(const Duration(hours: 2)),
        );

        await tester.pumpWidgetWithAppWrapper(
          MaintenanceScreen(maintenanceWindow: maintenance),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('single_day.dark');
      });
    });

    testGoldens('MaintenanceScreen Light Landscape - Single day', (tester) async {
      await withClock(Clock.fixed(DateTime(2025, 1, 15, 10, 30)), () async {
        final now = clock.now();
        final maintenance = MaintenanceWindow(
          startDateTime: now,
          endDateTime: now.add(const Duration(hours: 2)),
        );

        await tester.pumpWidgetWithAppWrapper(
          MaintenanceScreen(maintenanceWindow: maintenance),
          surfaceSize: iphoneXSizeLandscape,
        );
        await screenMatchesGolden('single_day.light.landscape');
      });
    });

    testGoldens('MaintenanceScreen Dark Landscape - Single day', (tester) async {
      await withClock(Clock.fixed(DateTime(2025, 1, 15, 10, 30)), () async {
        final now = clock.now();
        final maintenance = MaintenanceWindow(
          startDateTime: now,
          endDateTime: now.add(const Duration(hours: 2)),
        );

        await tester.pumpWidgetWithAppWrapper(
          MaintenanceScreen(maintenanceWindow: maintenance),
          brightness: Brightness.dark,
          surfaceSize: iphoneXSizeLandscape,
        );
        await screenMatchesGolden('single_day.dark.landscape');
      });
    });

    testGoldens('MaintenanceScreen Light Scaled Text - Single day', (tester) async {
      await withClock(Clock.fixed(DateTime(2025, 1, 15, 10, 30)), () async {
        final now = clock.now();
        final maintenance = MaintenanceWindow(
          startDateTime: now,
          endDateTime: now.add(const Duration(hours: 2)),
        );

        await tester.pumpWidgetWithAppWrapper(
          MaintenanceScreen(maintenanceWindow: maintenance),
          textScaleSize: 1.3,
        );
        await screenMatchesGolden('single_day.light.scaled_text');
      });
    });

    testGoldens('MaintenanceScreen Light - Multi day (Overnight)', (tester) async {
      await withClock(Clock.fixed(DateTime(2025, 1, 15, 22, 00)), () async {
        final today = clock.now();
        final tomorrow = today.add(const Duration(days: 1));
        final maintenance = MaintenanceWindow(
          startDateTime: today.add(const Duration(hours: 2)),
          endDateTime: tomorrow.add(const Duration(hours: 4)),
        );

        await tester.pumpWidgetWithAppWrapper(
          MaintenanceScreen(maintenanceWindow: maintenance),
        );
        await screenMatchesGolden('multi_day.light');
      });
    });

    testGoldens('MaintenanceScreen Dark - Multi day (Overnight)', (tester) async {
      await withClock(Clock.fixed(DateTime(2025, 1, 15, 22, 00)), () async {
        final today = clock.now();
        final tomorrow = today.add(const Duration(days: 1));
        final maintenance = MaintenanceWindow(
          startDateTime: today.add(const Duration(hours: 2)),
          endDateTime: tomorrow.add(const Duration(hours: 4)),
        );

        await tester.pumpWidgetWithAppWrapper(
          MaintenanceScreen(maintenanceWindow: maintenance),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('multi_day.dark');
      });
    });

    testGoldens('MaintenanceScreen Light Landscape - Multi day (Overnight)', (tester) async {
      await withClock(Clock.fixed(DateTime(2025, 1, 15, 22, 00)), () async {
        final today = clock.now();
        final tomorrow = today.add(const Duration(days: 1));
        final maintenance = MaintenanceWindow(
          startDateTime: today.add(const Duration(hours: 2)),
          endDateTime: tomorrow.add(const Duration(hours: 4)),
        );

        await tester.pumpWidgetWithAppWrapper(
          MaintenanceScreen(maintenanceWindow: maintenance),
          surfaceSize: iphoneXSizeLandscape,
        );
        await screenMatchesGolden('multi_day.light.landscape');
      });
    });

    testGoldens('MaintenanceScreen Dark Landscape - Multi day (Overnight)', (tester) async {
      await withClock(Clock.fixed(DateTime(2025, 1, 15, 22, 00)), () async {
        final today = clock.now();
        final tomorrow = today.add(const Duration(days: 1));
        final maintenance = MaintenanceWindow(
          startDateTime: today.add(const Duration(hours: 2)),
          endDateTime: tomorrow.add(const Duration(hours: 4)),
        );

        await tester.pumpWidgetWithAppWrapper(
          MaintenanceScreen(maintenanceWindow: maintenance),
          brightness: Brightness.dark,
          surfaceSize: iphoneXSizeLandscape,
        );
        await screenMatchesGolden('multi_day.dark.landscape');
      });
    });

    testGoldens('MaintenanceScreen Light Scaled Text - Multi day (Overnight)', (tester) async {
      await withClock(Clock.fixed(DateTime(2025, 1, 15, 22, 00)), () async {
        final today = clock.now();
        final tomorrow = today.add(const Duration(days: 1));
        final maintenance = MaintenanceWindow(
          startDateTime: today.add(const Duration(hours: 2)),
          endDateTime: tomorrow.add(const Duration(hours: 4)),
        );

        await tester.pumpWidgetWithAppWrapper(
          MaintenanceScreen(maintenanceWindow: maintenance),
          textScaleSize: 1.3,
        );
        await screenMatchesGolden('multi_day.light.scaled_text');
      });
    });
  });

  group('widgets', () {
    testWidgets('Title is displayed', (tester) async {
      await withClock(Clock.fixed(DateTime(2025, 1, 15, 10, 30)), () async {
        final now = clock.now();
        final maintenance = MaintenanceWindow(
          startDateTime: now,
          endDateTime: now.add(const Duration(hours: 2)),
        );

        await tester.pumpWidgetWithAppWrapper(
          MaintenanceScreen(maintenanceWindow: maintenance),
        );

        final l10n = await TestUtils.englishLocalizations;
        expect(find.text(l10n.maintenanceScreenHeadline), findsWidgets);
      });
    });

    testWidgets('Description is displayed for single day maintenance', (tester) async {
      await withClock(Clock.fixed(DateTime(2025, 1, 15, 10, 30)), () async {
        final now = clock.now();
        final maintenance = MaintenanceWindow(
          startDateTime: now,
          endDateTime: now.add(const Duration(hours: 2)),
        );

        await tester.pumpWidgetWithAppWrapper(
          MaintenanceScreen(maintenanceWindow: maintenance),
        );

        expect(find.byType(BodyText), findsWidgets);
      });
    });

    testWidgets('Description is displayed for multi-day maintenance', (tester) async {
      await withClock(Clock.fixed(DateTime(2025, 1, 15, 22, 00)), () async {
        final today = clock.now();
        final tomorrow = today.add(const Duration(days: 1));
        final maintenance = MaintenanceWindow(
          startDateTime: today.add(const Duration(hours: 2)),
          endDateTime: tomorrow.add(const Duration(hours: 4)),
        );

        await tester.pumpWidgetWithAppWrapper(
          MaintenanceScreen(maintenanceWindow: maintenance),
        );

        expect(find.byType(BodyText), findsWidgets);
      });
    });

    testWidgets('Help button is displayed', (tester) async {
      await withClock(Clock.fixed(DateTime(2025, 1, 15, 10, 30)), () async {
        final now = clock.now();
        final maintenance = MaintenanceWindow(
          startDateTime: now,
          endDateTime: now.add(const Duration(hours: 2)),
        );

        await tester.pumpWidgetWithAppWrapper(
          MaintenanceScreen(maintenanceWindow: maintenance),
        );

        expect(find.byType(HelpIconButton), findsOneWidget);
      });
    });
  });
}
