import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/dialog/locked_out_dialog.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    testGoldens(
      'Locked Out Dialog',
      (tester) async {
        final Key showDialogButton = Key('showDialogButton');
        await tester.pumpWidgetWithAppWrapper(
          Scaffold(
            body: Builder(
              builder: (context) {
                return Center(
                  child: TextButton(
                    onPressed: () => LockedOutDialog.show(context),
                    child: Text('Show Dialog', key: showDialogButton),
                  ),
                );
              },
            ),
          ),
        );
        await tester.tap(find.byKey(showDialogButton));
        await tester.pumpAndSettle();
        await screenMatchesGolden('locked_out_dialog');
      },
    );
  });
}
