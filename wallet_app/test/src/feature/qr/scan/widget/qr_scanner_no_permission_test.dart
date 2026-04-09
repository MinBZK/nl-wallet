import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/qr/scan/widget/qr_scanner_no_permission.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../test_util/test_utils.dart';

void main() {
  testWidgets('ltc7 ltc16 ltc19 a button to retry getting the required permission is shown', (
    WidgetTester tester,
  ) async {
    await tester.pumpWidgetWithAppWrapper(const QrScannerNoPermission(isPermanentlyDenied: false));

    final l10n = await TestUtils.englishLocalizations;
    expect(find.text(l10n.qrScanTabGrantPermissionCta), findsOneWidget);
  });
}
