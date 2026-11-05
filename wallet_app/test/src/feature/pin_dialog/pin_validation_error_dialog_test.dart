import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';
import 'package:wallet/src/feature/pin_dialog/pin_validation_error_dialog.dart';

import '../../test_util/dialog_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  for (final reason in PinValidationError.values) {
    testWidgets('PinValidationErrorDialog ($reason) exposes its title as heading', (tester) async {
      await DialogUtils.pumpDialog(tester, (context) => PinValidationErrorDialog.show(context, reason));

      final l10n = await TestUtils.englishLocalizations;
      TestUtils.expectHeading(tester, l10n.pinValidationErrorDialogTitle);
    });
  }
}
