import 'package:flutter/cupertino.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:wallet/src/domain/model/permission/permission_check_result.dart';
import 'package:wallet/src/feature/common/widget/utility/check_permissions_on_resume.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockCheckPermissionUseCase mockCheckPermissionUseCase;

  setUp(() {
    mockCheckPermissionUseCase = MockCheckPermissionUseCase();
  });

  testWidgets(
    'onPermissionGranted is called when the permission is granted and the lifecycle moves to resumed',
    (tester) async {
      bool? granted;
      when(mockCheckPermissionUseCase.invoke(any)).thenAnswer(
        (_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false),
      );
      await tester.pumpWidget(
        CheckPermissionsOnResume(
          onPermissionGranted: () => granted = true,
          onPermissionDenied: () => granted = false,
          permissions: [Permission.camera],
          checkPermissionUseCase: mockCheckPermissionUseCase,
          child: const Placeholder(),
        ),
      );

      final binding = TestWidgetsFlutterBinding.ensureInitialized();
      binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pumpAndSettle(); // Make sure lifecycle state is handled
      expect(granted, isTrue);
    },
  );

  testWidgets(
    'onPermissionGranted is not called when the permission is not granted and the lifecycle moves to resumed',
    (tester) async {
      bool? granted;
      when(mockCheckPermissionUseCase.invoke(any)).thenAnswer(
        (_) async => const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false),
      );
      await tester.pumpWidget(
        CheckPermissionsOnResume(
          onPermissionGranted: () => granted = true,
          onPermissionDenied: () => granted = false,
          permissions: [Permission.camera, Permission.bluetoothAdvertise],
          checkPermissionUseCase: mockCheckPermissionUseCase,
          child: const Placeholder(),
        ),
      );

      final binding = TestWidgetsFlutterBinding.ensureInitialized();
      binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pumpAndSettle(); // Make sure lifecycle state is handled
      expect(granted, isFalse);
    },
  );
}
