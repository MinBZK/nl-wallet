import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/dialog/idle_warning_dialog.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    testGoldens(
      'Timeout Dialog',
      (tester) async {
        final Key showDialogButton = const Key('showDialogButton');
        await tester.pumpWidgetWithAppWrapper(
          Scaffold(
            body: Builder(
              builder: (context) {
                return Center(
                  child: TextButton(
                    onPressed: () => IdleWarningDialog.show(context),
                    child: Text('Show Dialog', key: showDialogButton),
                  ),
                );
              },
            ),
          ),
        );
        await tester.tap(find.byKey(showDialogButton));
        await tester.pumpAndSettle();
        await screenMatchesGolden('idle_warning_dialog');
      },
    );
  });
}
