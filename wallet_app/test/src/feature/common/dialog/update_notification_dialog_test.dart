import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/dialog/update_notification_dialog.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    testGoldens(
      'Update Notification Dialog',
      (tester) async {
        final Key showDialogButton = Key('showDialogButton');
        await tester.pumpWidgetWithAppWrapper(
          Scaffold(
            body: Builder(
              builder: (context) {
                return Center(
                  child: TextButton(
                    onPressed: () => UpdateNotificationDialog.show(context),
                    child: Text('Show Dialog', key: showDialogButton),
                  ),
                );
              },
            ),
          ),
        );
        await tester.tap(find.byKey(showDialogButton));
        await tester.pumpAndSettle();
        await screenMatchesGolden('update_notification_dialog');
      },
    );

    testGoldens(
      'Update Notification Dialog - 3 hours left',
      (tester) async {
        final Key showDialogButton = Key('showDialogButton');
        await tester.pumpWidgetWithAppWrapper(
          Scaffold(
            body: Builder(
              builder: (context) {
                return Center(
                  child: TextButton(
                    onPressed: () => UpdateNotificationDialog.show(context, timeUntilBlocked: Duration(hours: 3)),
                    child: Text('Show Dialog', key: showDialogButton),
                  ),
                );
              },
            ),
          ),
        );
        await tester.tap(find.byKey(showDialogButton));
        await tester.pumpAndSettle();
        await screenMatchesGolden('update_notification_dialog.3_hours');
      },
    );
  });
}
