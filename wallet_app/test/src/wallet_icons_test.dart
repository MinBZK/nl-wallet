import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/wallet_icons.dart';

import '../wallet_app_test_widget.dart';
import 'test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('face id icon renders as expected', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const Icon(WalletIcons.icon_face_id, size: 32),
        surfaceSize: const Size(32, 32),
      );
      await screenMatchesGolden('icons/face_id');
    });
  });
}
