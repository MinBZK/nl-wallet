import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/button/animated_visibility_back_button.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../test_util/golden_utils.dart';

void main() {
  const kGoldenSize = Size(50, 50);

  group('goldens', () {
    testGoldens('back button visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AnimatedVisibilityBackButton(visible: true),
        surfaceSize: kGoldenSize,
      );
      await screenMatchesGolden('animated_visibility_back_button/light');
    });
    testGoldens('back button invisible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AnimatedVisibilityBackButton(visible: false),
        surfaceSize: kGoldenSize,
      );
      await screenMatchesGolden('animated_visibility_back_button/light.invisible');
    });
    testGoldens('back button dark visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AnimatedVisibilityBackButton(visible: true),
        brightness: Brightness.dark,
        surfaceSize: kGoldenSize,
      );
      await screenMatchesGolden('animated_visibility_back_button/dark');
    });
    testGoldens('back button dark invisible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AnimatedVisibilityBackButton(visible: false),
        brightness: Brightness.dark,
        surfaceSize: kGoldenSize,
      );
      await screenMatchesGolden('animated_visibility_back_button/dark.invisible');
    });
  });
}
