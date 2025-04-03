import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/animated_linear_progress_indicator.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  const kGoldenSize = Size(200, 4);

  group('goldens', () {
    testGoldens(
      'light animated linear progress @ 0.25',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const AnimatedLinearProgressIndicator(progress: 0.25),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('animated_linear_progress_indicator/light');
      },
    );
    testGoldens(
      'light animated linear progress @ 0.75',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const AnimatedLinearProgressIndicator(progress: 0.75),
          brightness: Brightness.dark,
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('animated_linear_progress_indicator/dark');
      },
    );
  });
}
