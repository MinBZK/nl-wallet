import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/utility/do_on_init.dart';

import '../../../../../wallet_app_test_widget.dart';

void main() {
  testWidgets('on init is called', (tester) async {
    bool onInitCalled = false;
    await tester.pumpWidgetWithAppWrapper(
      DoOnInit(
        child: const SizedBox.shrink(),
        onInit: (c) => onInitCalled = true,
      ),
    );

    expect(onInitCalled, isTrue);
  });
}
