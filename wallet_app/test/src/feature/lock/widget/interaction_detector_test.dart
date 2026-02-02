import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/lock/widget/interaction_detector.dart';

import '../../../../wallet_app_test_widget.dart';

void main() {
  testWidgets('InteractionDetector calls onInteraction when hit', (WidgetTester tester) async {
    bool interacted = false;

    await tester.pumpWidgetWithAppWrapper(
      InteractionDetector(
        onInteraction: () => interacted = true,
        child: const SizedBox(width: 100, height: 100),
      ),
      surfaceSize: const Size(100, 100),
    );

    // Simulate a tap on the InteractionDetector
    await tester.tapAt(const Offset(50, 50));

    // Verify that onInteraction was called
    expect(interacted, isTrue);
  });
}
