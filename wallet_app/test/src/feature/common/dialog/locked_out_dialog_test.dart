import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/dialog/locked_out_dialog.dart';

import '../../../test_util/dialog_utils.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    testGoldens(
      'Locked Out Dialog',
      (tester) async {
        await DialogUtils.pumpDialog(tester, LockedOutDialog.show);
        await screenMatchesGolden('locked_out_dialog');
      },
    );
  });
}
