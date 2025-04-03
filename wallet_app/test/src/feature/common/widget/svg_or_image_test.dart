import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/svg_or_image.dart';
import 'package:wallet/src/wallet_assets.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    testGoldens(
      'svg is rendered as expected',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const SvgOrImage(asset: WalletAssets.svg_rijks_card_bg_light),
          surfaceSize: const Size(1001, 2000),
        );
        await screenMatchesGolden('svg_or_image/svg');
      },
    );
    testGoldens(
      'png is rendered as expected',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const SvgOrImage(asset: WalletAssets.logo_card_rijksoverheid),
          surfaceSize: const Size(40, 40),
        );
        await screenMatchesGolden('svg_or_image/png');
      },
    );
  });
}
