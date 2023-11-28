import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/widget/utility/limit_font_scaling.dart';

import '../../../../../wallet_app_test_widget.dart';

void main() {
  group('goldens', () {
    testGoldens(
      'light limit font scaling',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const MediaQuery(
            data: MediaQueryData(textScaler: TextScaler.linear(3.0)),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Not limited (3.0)'),
                LimitFontScaling(maxScaleFactor: 1.0, child: Text('Limited (1.0)')),
              ],
            ),
          ),
          surfaceSize: const Size(300, 79),
        );
        await screenMatchesGolden(tester, 'limit_font_scaling/light');
      },
    );
  });
}
