import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/wallet_transfer_faq/wallet_transfer_faq_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('ltc62 Faq Screen - Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const WalletTransferFaqScreen(), surfaceSize: iphoneXSize);
      await screenMatchesGolden('faq.light');
    });

    testGoldens('ltc62 Faq Screen - Dark - Landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferFaqScreen(),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('faq.dark.landscape');
    });
  });
}
