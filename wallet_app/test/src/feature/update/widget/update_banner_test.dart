import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/update/observe_version_state_usecase.dart';
import 'package:wallet/src/feature/update/widget/update_banner.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.mocks.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

const updateBannerSize = Size(380, 80);

void main() {
  Future<void> pumpUpdateBanner(
    WidgetTester tester, {
    required VersionState state,
    Brightness brightness = Brightness.light,
    Size surfaceSize = updateBannerSize,
    double textScaleSize = 1.0,
  }) async {
    await tester.pumpWidgetWithAppWrapper(
      const UpdateBanner(),
      surfaceSize: surfaceSize,
      providers: [
        RepositoryProvider<ObserveVersionStateUsecase>(
          create: (c) {
            final usecase = MockObserveVersionStateUsecase();
            when(usecase.invoke()).thenAnswer((_) => Stream.value(state));
            return usecase;
          },
        ),
      ],
      brightness: brightness,
      textScaleSize: textScaleSize,
    );
    await tester.pumpAndSettle();
  }

  group('goldens', () {
    testGoldens(
      'VersionStateOk',
      (tester) async {
        // Should be empty, so drawing it on 1x1 should not throw errors.
        await pumpUpdateBanner(tester, state: VersionStateOk(), surfaceSize: const Size(1, 1));
        await screenMatchesGolden('ok.light');
      },
    );
    testGoldens(
      'VersionStateNotify',
      (tester) async {
        await pumpUpdateBanner(tester, state: VersionStateNotify());
        await screenMatchesGolden('notify.light');
      },
    );
    testGoldens(
      'VersionStateRecommend',
      (tester) async {
        await pumpUpdateBanner(tester, state: VersionStateRecommend());
        await screenMatchesGolden('recommend.light');
      },
    );

    testGoldens(
      'VersionStateWarn 2 weeks',
      (tester) async {
        await pumpUpdateBanner(tester, state: VersionStateWarn(timeUntilBlocked: const Duration(days: 14)));
        await screenMatchesGolden('warn.2_weeks.light');
      },
    );

    testGoldens(
      'VersionStateWarn 2 weeks dark',
      (tester) async {
        await pumpUpdateBanner(
          tester,
          state: VersionStateWarn(timeUntilBlocked: const Duration(days: 14)),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('warn.2_weeks.dark');
      },
    );

    testGoldens(
      'VersionStateWarn 3 days',
      (tester) async {
        await pumpUpdateBanner(tester, state: VersionStateWarn(timeUntilBlocked: const Duration(days: 3)));
        await screenMatchesGolden('warn.3_days.light');
      },
    );

    testGoldens(
      'VersionStateWarn 30 mins',
      (tester) async {
        await pumpUpdateBanner(tester, state: VersionStateWarn(timeUntilBlocked: const Duration(minutes: 30)));
        await screenMatchesGolden('warn.30_mins.light');
      },
    );

    testGoldens(
      'VersionStateBlock',
      (tester) async {
        // Should be empty, so drawing it on 1x1 should not throw errors.
        await pumpUpdateBanner(tester, state: VersionStateBlock(), surfaceSize: const Size(1, 1));
        await screenMatchesGolden('blocked.light');
      },
    );

    testGoldens(
      'light notify focused',
      (tester) async {
        await pumpUpdateBanner(tester, state: VersionStateNotify());
        await tester.sendKeyEvent(LogicalKeyboardKey.tab); // Trigger focused state
        await tester.pumpAndSettle();
        await screenMatchesGolden('notify.focused.light');
      },
    );

    testGoldens(
      'dark focused',
      (tester) async {
        await pumpUpdateBanner(tester, state: VersionStateNotify(), brightness: Brightness.dark);
        await screenMatchesGolden('notify.dark');
      },
    );

    testGoldens(
      'dark notify focused',
      (tester) async {
        await pumpUpdateBanner(tester, state: VersionStateNotify(), brightness: Brightness.dark);
        await tester.sendKeyEvent(LogicalKeyboardKey.tab); // Trigger focused state
        await tester.pumpAndSettle();
        await screenMatchesGolden('notify.focused.dark');
      },
    );

    testGoldens(
      'VersionStateWarn 2 hours - Scaled 2x',
      (tester) async {
        await pumpUpdateBanner(
          tester,
          state: VersionStateWarn(timeUntilBlocked: const Duration(hours: 2)),
          textScaleSize: 2,
          brightness: Brightness.dark,
          surfaceSize: const Size(380, 220),
        );
        await screenMatchesGolden('warn.2_hours.scaled.dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('VersionStateNotify banner shows notify title', (tester) async {
      await pumpUpdateBanner(tester, state: VersionStateNotify());
      final l10n = await TestUtils.englishLocalizations;
      final widgetFinder = find.text(l10n.updateBannerNotifyTitle);
      expect(widgetFinder, findsOneWidget);
    });

    testWidgets('VersionStateRecommend banner shows recommend title', (tester) async {
      await pumpUpdateBanner(tester, state: VersionStateRecommend());
      final l10n = await TestUtils.englishLocalizations;
      final widgetFinder = find.text(l10n.updateBannerRecommendTitle);
      expect(widgetFinder, findsOneWidget);
    });

    testWidgets('VersionStateWarn banner shows warn title', (tester) async {
      await pumpUpdateBanner(tester, state: VersionStateWarn(timeUntilBlocked: const Duration(days: 1)));
      final l10n = await TestUtils.englishLocalizations;
      final widgetFinder = find.text(l10n.updateBannerWarnTitle);
      expect(widgetFinder, findsOneWidget);
    });
  });
}
