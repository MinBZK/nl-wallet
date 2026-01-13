import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/dialog/idle_warning_dialog.dart';

import '../../../test_util/dialog_utils.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    testGoldens(
      'Timeout Dialog',
      (tester) async {
        await DialogUtils.pumpDialog(tester, IdleWarningDialog.show);
        await screenMatchesGolden('idle_warning_dialog');
      },
    );
  });
}
