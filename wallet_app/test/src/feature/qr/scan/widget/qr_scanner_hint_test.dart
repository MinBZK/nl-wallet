import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/qr/scan/widget/qr_scanner_hint.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../test_util/test_utils.dart';

void main() {
  testWidgets('QrScannerHint exposes the scan instruction as heading', (tester) async {
    await tester.pumpWidgetWithAppWrapper(const QrScannerHint());

    final l10n = await TestUtils.englishLocalizations;
    TestUtils.expectHeading(tester, l10n.qrScreenScanHint);
  });
}
