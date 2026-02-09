import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/permission/permission_check_result.dart';
import 'package:wallet/src/domain/usecase/notification/set_push_notifications_setting_usecase.dart';
import 'package:wallet/src/domain/usecase/permission/request_permission_usecase.dart';
import 'package:wallet/src/feature/notification/sheet/request_notification_permission_sheet.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

void main() {
  late MockRequestPermissionUseCase requestPermissionUseCase;
  late MockSetPushNotificationsSettingUseCase setPushNotificationsSettingUseCase;

  setUp(() {
    requestPermissionUseCase = MockRequestPermissionUseCase();
    setPushNotificationsSettingUseCase = MockSetPushNotificationsSettingUseCase();
  });

  group('goldens', () {
    testGoldens(
      'light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const RequestNotificationPermissionSheet().withDependency<RequestPermissionUseCase>(
            (c) => requestPermissionUseCase,
          ),
          surfaceSize: const Size(350, 423),
        );
        await tester.pumpAndSettle();
        await screenMatchesGolden('request_notification_permission_sheet/light');
      },
    );

    testGoldens(
      'dark - landscape (tests button reflow)',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const RequestNotificationPermissionSheet().withDependency<RequestPermissionUseCase>(
            (c) => requestPermissionUseCase,
          ),
          surfaceSize: const Size(450, 305),
          brightness: Brightness.dark,
        );
        await tester.pumpAndSettle();
        await screenMatchesGolden('request_notification_permission_sheet/dark.landscape');
      },
    );
  });

  group('actions', () {
    testWidgets(
      'tapping allow in dialog triggers the permission usecase, when permission is granted the setting is set to true',
      (
        tester,
      ) async {
        when(
          requestPermissionUseCase.invoke(.notification),
        ).thenAnswer((_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));

        await tester.pumpWidgetWithAppWrapper(
          const RequestNotificationPermissionSheet(),
          providers: [
            RepositoryProvider<RequestPermissionUseCase>.value(value: requestPermissionUseCase),
            RepositoryProvider<SetPushNotificationsSettingUseCase>.value(value: setPushNotificationsSettingUseCase),
          ],
        );

        final l10n = await TestUtils.englishLocalizations;
        final allowButtonFinder = find.text(l10n.requestNotificationPermissionSheetPositiveCta);
        expect(allowButtonFinder, findsOneWidget);
        await tester.tap(allowButtonFinder);
        await tester.pump();

        // Verify that permission is requested
        verify(requestPermissionUseCase.invoke(.notification)).called(1);
        // Verify the user setting is set when permission is granted
        verify(setPushNotificationsSettingUseCase.invoke(enabled: true)).called(1);
      },
    );

    testWidgets(
      'tapping allow in dialog triggers the permission usecase, when permission is NOT granted the setting is set to false',
      (
        tester,
      ) async {
        when(
          requestPermissionUseCase.invoke(.notification),
        ).thenAnswer((_) async => const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false));

        await tester.pumpWidgetWithAppWrapper(
          const RequestNotificationPermissionSheet(),
          providers: [
            RepositoryProvider<RequestPermissionUseCase>.value(value: requestPermissionUseCase),
            RepositoryProvider<SetPushNotificationsSettingUseCase>.value(value: setPushNotificationsSettingUseCase),
          ],
        );

        final l10n = await TestUtils.englishLocalizations;
        final allowButtonFinder = find.text(l10n.requestNotificationPermissionSheetPositiveCta);
        expect(allowButtonFinder, findsOneWidget);
        await tester.tap(allowButtonFinder);
        await tester.pump();

        // Verify that permission is requested
        verify(requestPermissionUseCase.invoke(.notification)).called(1);
        // Verify the user setting is set when permission is granted
        verify(setPushNotificationsSettingUseCase.invoke(enabled: false)).called(1);
      },
    );

    testWidgets('tapping dismiss in dialog does not trigger the permission request usecase', (
      tester,
    ) async {
      when(
        requestPermissionUseCase.invoke(.notification),
      ).thenAnswer((_) async => const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false));

      await tester.pumpWidgetWithAppWrapper(
        const RequestNotificationPermissionSheet(),
        providers: [
          RepositoryProvider<RequestPermissionUseCase>.value(value: requestPermissionUseCase),
          RepositoryProvider<SetPushNotificationsSettingUseCase>.value(value: setPushNotificationsSettingUseCase),
        ],
      );

      final l10n = await TestUtils.englishLocalizations;
      final allowButtonFinder = find.text(l10n.requestNotificationPermissionSheetNegativeCta);
      expect(allowButtonFinder, findsOneWidget);
      await tester.tap(allowButtonFinder);
      await tester.pump();

      // Verify that permission is NOT requested
      verifyNever(requestPermissionUseCase.invoke(.notification));
      // Verify the user setting is NOT set when permission is granted
      verifyNever(setPushNotificationsSettingUseCase.invoke(enabled: anyNamed('enabled')));
    });
  });
}
