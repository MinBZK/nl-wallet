import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:wallet/src/domain/model/permission/permission_check_result.dart';
import 'package:wallet/src/domain/usecase/permission/request_permission_usecase.dart';
import 'package:wallet/src/feature/common/sheet/qr_action_sheet.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.mocks.dart';
import '../../../test_util/test_utils.dart';

void main() {
  late MockRequestPermissionUseCase requestPermissionUseCase;

  setUp(() {
    requestPermissionUseCase = MockRequestPermissionUseCase();
  });

  /// Pumps a [Scaffold] that shows the [QrActionSheet] as a bottom sheet,
  /// so that [Navigator.pop] works correctly within the sheet.
  Future<void> pumpSheetInContext(WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      Scaffold(
        body: Builder(
          builder: (context) {
            return Center(
              child: TextButton(
                key: const Key('showSheet'),
                onPressed: () => QrActionSheet.show(context),
                child: const Text('Show'),
              ),
            );
          },
        ),
      ),
      providers: [
        RepositoryProvider<RequestPermissionUseCase>.value(value: requestPermissionUseCase),
      ],
    );
    await tester.tap(find.byKey(const Key('showSheet')));
    await tester.pumpAndSettle();
  }

  group('show QR permission flow', () {
    testWidgets('tapping show QR requests bluetooth permission', (tester) async {
      when(requestPermissionUseCase.invoke([Permission.bluetooth])).thenAnswer(
        (_) async => const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false),
      );

      await pumpSheetInContext(tester);

      final l10n = await TestUtils.englishLocalizations;
      await tester.tap(find.text(l10n.qrActionSheetShowQrTitle));
      await tester.pumpAndSettle();

      verify(requestPermissionUseCase.invoke([Permission.bluetooth])).called(1);
    });

    testWidgets('shows BlePermissionDialog when permission is permanently denied', (tester) async {
      when(requestPermissionUseCase.invoke([Permission.bluetooth])).thenAnswer(
        (_) async => const PermissionCheckResult(isGranted: false, isPermanentlyDenied: true),
      );

      await pumpSheetInContext(tester);

      final l10n = await TestUtils.englishLocalizations;
      await tester.tap(find.text(l10n.qrActionSheetShowQrTitle));
      await tester.pumpAndSettle();

      // Verify the BlePermissionDialog is shown
      expect(find.text(l10n.qrShowBluetoothPermissionTitle), findsOneWidget);
      expect(find.text(l10n.qrShowBluetoothPermissionDescription), findsOneWidget);
    });

    testWidgets('does not show dialog when permission is denied but not permanently', (tester) async {
      when(requestPermissionUseCase.invoke([Permission.bluetooth])).thenAnswer(
        (_) async => const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false),
      );

      await pumpSheetInContext(tester);

      final l10n = await TestUtils.englishLocalizations;
      await tester.tap(find.text(l10n.qrActionSheetShowQrTitle));
      await tester.pumpAndSettle();

      // Verify no dialog is shown
      expect(find.text(l10n.qrShowBluetoothPermissionTitle), findsNothing);
    });
  });
}
