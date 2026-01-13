import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/dialog/generic_dialog.dart';

import '../../../test_util/dialog_utils.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  group('GenericDialog Goldens', () {
    testGoldens('Active Disclosure Session Dialog', (tester) async {
      await DialogUtils.pumpDialog(tester, GenericDialog.showActiveDisclosureSession);
      await screenMatchesGolden('generic/dialog_active_disclosure_session');
    });

    testGoldens('Active Issuance Session Dialog', (tester) async {
      await DialogUtils.pumpDialog(tester, GenericDialog.showActiveIssuanceSession);
      await screenMatchesGolden('generic/dialog_active_issuance_session');
    });

    testGoldens('Finish Setup Dialog', (tester) async {
      await DialogUtils.pumpDialog(tester, GenericDialog.showFinishSetup);
      await screenMatchesGolden('generic/dialog_finish_setup');
    });

    testGoldens('Finish Transfer Dialog', (tester) async {
      await DialogUtils.pumpDialog(tester, GenericDialog.showFinishTransfer);
      await screenMatchesGolden('generic/dialog_finish_transfer');
    });

    testGoldens('Finish Pin Dialog', (tester) async {
      await DialogUtils.pumpDialog(tester, GenericDialog.showFinishPin);
      await screenMatchesGolden('generic/dialog_finish_pin');
    });

    testGoldens('Generic Show Dialog', (tester) async {
      await DialogUtils.pumpDialog(
        tester,
        (context) => GenericDialog.show(
          context,
          title: 'Custom Title',
          description: 'Custom Description',
        ),
      );
      await screenMatchesGolden('generic/dialog_custom');
    });
  });
}
