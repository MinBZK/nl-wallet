import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/dialog/update_notification_dialog.dart';

import '../../../test_util/dialog_utils.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    testGoldens(
      'Update Notification Dialog',
      (tester) async {
        await DialogUtils.pumpDialog(tester, UpdateNotificationDialog.show);
        await screenMatchesGolden('update_notification_dialog');
      },
    );

    testGoldens(
      'Update Notification Dialog - 3 hours left',
      (tester) async {
        await DialogUtils.pumpDialog(
          tester,
          (context) {
            return UpdateNotificationDialog.show(
              context,
              timeUntilBlocked: const Duration(hours: 3),
            );
          },
        );
        await screenMatchesGolden('update_notification_dialog.3_hours');
      },
    );
  });
}
