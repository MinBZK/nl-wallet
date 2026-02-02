import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/permission/request_permission_usecase.dart';
import 'package:wallet/src/feature/notification/sheet/request_notification_permission_sheet.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

void main() {
  late MockRequestPermissionUseCase requestPermissionUseCase;

  setUp(() {
    requestPermissionUseCase = MockRequestPermissionUseCase();
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
    testWidgets('tapping allow calls use case', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RequestNotificationPermissionSheet(),
        providers: [RepositoryProvider<RequestPermissionUseCase>.value(value: requestPermissionUseCase)],
      );

      final l10n = await TestUtils.englishLocalizations;
      final allowButtonFinder = find.text(l10n.requestNotificationPermissionSheetPositiveCta);
      expect(allowButtonFinder, findsOneWidget);
      await tester.tap(allowButtonFinder);
      await tester.pump();

      verify(requestPermissionUseCase.invoke(.notification)).called(1);
    });
  });
}
