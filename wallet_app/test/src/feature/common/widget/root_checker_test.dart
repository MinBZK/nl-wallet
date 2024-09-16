import 'package:flutter/cupertino.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/root/root_checker.dart';

import '../../../../wallet_app_test_widget.dart';

void main() {
  group('widgets', () {
    testWidgets('the child is rendered when no root is detected', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        RootChecker(
          child: const Text('no root'),
          customRootCheck: (c) async => false,
        ),
      );

      // No root detected, the child should be shown as normal
      expect(find.text('no root'), findsOneWidget);
    });

    testWidgets('the child is NOT rendered when root is detected', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        RootChecker(
          child: const Text('no root'),
          customRootCheck: (c) async => true,
        ),
      );

      // Root detected, the child should not be shown
      expect(find.text('no root'), findsNothing);
    });

    testWidgets('root is re-evaluated on resume', (tester) async {
      int rootCheckCount = 0;
      await tester.pumpWidgetWithAppWrapper(
        RootChecker(
          child: const Text('no root'),
          customRootCheck: (c) async {
            rootCheckCount++;
            return false;
          },
        ),
      );

      // Initial check performed
      expect(rootCheckCount, 1);

      TestWidgetsFlutterBinding.instance.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      TestWidgetsFlutterBinding.instance.handleAppLifecycleStateChanged(AppLifecycleState.resumed);

      // onResume check is performed
      expect(rootCheckCount, 2);
    });
  });
}
