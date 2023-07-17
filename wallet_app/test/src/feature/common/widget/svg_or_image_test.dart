import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/widget/svg_or_image.dart';

import '../../../../wallet_app_test_widget.dart';

void main() {
  group('goldens', () {
    testGoldens(
      'svg is rendered as expected',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const SvgOrImage(asset: 'assets/svg/rijks_card_bg_light.svg'),
          surfaceSize: const Size(1001, 2000),
        );
        await screenMatchesGolden(tester, 'svg_or_image/svg');
      },
    );
    testGoldens(
      'png is rendered as expected',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const SvgOrImage(asset: 'assets/non-free/images/logo_card_rijksoverheid.png'),
          surfaceSize: const Size(40, 40),
        );
        await screenMatchesGolden(tester, 'svg_or_image/png');
      },
    );
  });
}
