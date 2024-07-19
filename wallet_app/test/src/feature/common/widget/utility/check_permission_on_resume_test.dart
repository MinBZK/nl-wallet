import 'package:flutter/cupertino.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:wallet/src/feature/common/widget/utility/check_permission_on_resume.dart';

void main() {
  testWidgets(
    'onPermissionGranted is called when the permission is granted and the lifecycle moves to resumed',
    (tester) async {
      bool granted = false;
      await tester.pumpWidget(
        CheckPermissionOnResume(
          onPermissionGranted: () => granted = true,
          permission: Permission.camera,
          checkPermission: (permission) async => true,
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
      bool granted = false;
      await tester.pumpWidget(
        CheckPermissionOnResume(
          onPermissionGranted: () => granted = true,
          permission: Permission.camera,
          checkPermission: (permission) async => false,
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
