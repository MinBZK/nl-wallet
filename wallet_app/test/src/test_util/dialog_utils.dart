import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../wallet_app_test_widget.dart';

class DialogUtils {
  DialogUtils._();

  static Future<void> pumpDialog(WidgetTester tester, Future<void> Function(BuildContext) showDialog) async {
    const Key showDialogButton = Key('showDialogButton');
    await tester.pumpWidgetWithAppWrapper(
      Scaffold(
        body: Builder(
          builder: (context) {
            return Center(
              child: TextButton(
                onPressed: () => showDialog(context),
                child: const Text('Show Dialog', key: showDialogButton),
              ),
            );
          },
        ),
      ),
    );
    await tester.tap(find.byKey(showDialogButton));
    await tester.pumpAndSettle();
  }
}
