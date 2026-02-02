import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/card/preview/card_preview_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('ltc5 ltc25 CardPreview - light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardPreviewScreen(card: WalletMockData.card),
      );
      await screenMatchesGolden('card_preview.light');
    });

    testGoldens('ltc5 ltc25 CardPreview - dark, landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardPreviewScreen(card: WalletMockData.altCard),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('card_preview.dark.landscape');
    });
  });
}
