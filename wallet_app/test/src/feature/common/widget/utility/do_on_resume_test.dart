import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/utility/do_on_resume.dart';

void main() {
  testWidgets('DoOnResume calls onResume when lifecycle changes to resumed', (tester) async {
    int callCount = 0;
    await tester.pumpWidget(
      DoOnResume(
        onResume: () => callCount++,
        child: const SizedBox.shrink(),
      ),
    );

    final binding = TestWidgetsFlutterBinding.ensureInitialized();

    // Initial state in tests is usually resumed.
    // Transition to paused first.
    binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    await tester.pump();
    expect(callCount, 0);

    // Transition to resumed should trigger the callback.
    binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pump();
    expect(callCount, 1);

    // Transition back to resumed from inactive should also trigger.
    binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
    await tester.pump();
    binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pump();
    expect(callCount, 2);
  });

  testWidgets('DoOnResume does not call onResume for other lifecycle states', (tester) async {
    int callCount = 0;
    await tester.pumpWidget(
      DoOnResume(
        onResume: () => callCount++,
        child: const SizedBox.shrink(),
      ),
    );

    final binding = TestWidgetsFlutterBinding.ensureInitialized();

    binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
    await tester.pump();
    binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    await tester.pump();
    binding.handleAppLifecycleStateChanged(AppLifecycleState.detached);
    await tester.pump();

    expect(callCount, 0);
  });

  testWidgets('DoOnResume renders its child', (tester) async {
    const childKey = Key('child');
    await tester.pumpWidget(
      const DoOnResume(
        onResume: _noop,
        child: SizedBox(key: childKey),
      ),
    );

    expect(find.byKey(childKey), findsOneWidget);
  });

  testWidgets('DoOnResume does not trigger when it starts out as resumed', (tester) async {
    final binding = TestWidgetsFlutterBinding.ensureInitialized();
    binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);

    int callCount = 0;
    await tester.pumpWidget(
      DoOnResume(
        onResume: () => callCount++,
        child: const SizedBox.shrink(),
      ),
    );

    expect(callCount, 0);
  });
}

void _noop() {}
