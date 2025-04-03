import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/centered_loading_indicator.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  const kGoldenSize = Size(200, 200);

  group('goldens', () {
    testGoldens(
      'light centered loading indicator',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const CenteredLoadingIndicator(),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('centered_loading_indicator/light');
      },
    );
    testGoldens(
      'dark centered loading indicator',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const CenteredLoadingIndicator(),
          brightness: Brightness.dark,
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('centered_loading_indicator/dark');
      },
    );
  });
}
