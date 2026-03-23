import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/dialog/ble_permission_dialog.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/dialog_utils.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

void main() {
  testWidgets('BlePermissionDialog shows expected copy', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      const BlePermissionDialog(),
    );

    final l10n = await TestUtils.englishLocalizations;

    final titleFinder = find.text(l10n.qrShowBluetoothPermissionTitle);
    final descriptionFinder = find.text(l10n.qrShowBluetoothPermissionDescription);
    final closeFinder = find.text(l10n.generalDialogCloseCta.toUpperCase(), findRichText: true);
    final settingsFinder = find.text(l10n.qrShowBluetoothPermissionSettingsCta.toUpperCase(), findRichText: true);

    expect(titleFinder, findsOneWidget);
    expect(descriptionFinder, findsOneWidget);
    expect(closeFinder, findsOneWidget);
    expect(settingsFinder, findsOneWidget);
  });

  group('goldens', () {
    testGoldens(
      'BlePermissionDialog',
      (tester) async {
        await DialogUtils.pumpDialog(tester, BlePermissionDialog.show);
        await screenMatchesGolden('ble_permission_dialog');
      },
    );
  });
}
