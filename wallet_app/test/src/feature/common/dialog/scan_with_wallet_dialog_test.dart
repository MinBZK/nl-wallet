import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/dialog/scan_with_wallet_dialog.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/dialog_utils.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

void main() {
  testWidgets('ScanWithWalletDialog shows expected copy', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      const ScanWithWalletDialog(),
    );

    final l10n = await TestUtils.englishLocalizations;

    // Setup finders
    final titleFinder = find.text(l10n.scanWithWalletDialogTitle, findRichText: true);
    final descriptionFinder = find.text(l10n.scanWithWalletDialogBody, findRichText: true);
    final ctaFinder = find.text(l10n.scanWithWalletDialogScanCta.toUpperCase(), findRichText: true);

    // Verify all expected widgets show up once
    expect(titleFinder, findsOneWidget);
    expect(descriptionFinder, findsOneWidget);
    expect(ctaFinder, findsOneWidget);
  });

  group('goldens', () {
    testGoldens(
      'Scan with Wallet Dialog',
      (tester) async {
        await DialogUtils.pumpDialog(tester, ScanWithWalletDialog.show);
        await screenMatchesGolden('scan_with_wallet_dialog');
      },
    );
  });
}
