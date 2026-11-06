import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/pin_dialog/pin_confirmation_error_dialog.dart';

import '../../test_util/dialog_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  testWidgets('PinConfirmationErrorDialog exposes its title as heading', (tester) async {
    await DialogUtils.pumpDialog(tester, (context) => PinConfirmationErrorDialog.show(context, retryAllowed: true));

    final l10n = await TestUtils.englishLocalizations;
    TestUtils.expectHeading(tester, l10n.pinConfirmationErrorDialogTitle);
  });

  testWidgets('PinConfirmationErrorDialog (fatal) exposes its title as heading', (tester) async {
    await DialogUtils.pumpDialog(tester, (context) => PinConfirmationErrorDialog.show(context, retryAllowed: false));

    final l10n = await TestUtils.englishLocalizations;
    TestUtils.expectHeading(tester, l10n.pinConfirmationErrorDialogFatalTitle);
  });
}
