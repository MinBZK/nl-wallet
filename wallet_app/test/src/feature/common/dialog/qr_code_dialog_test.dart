import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/dialog/qr_code_dialog.dart';

import '../../../test_util/dialog_utils.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens(
      'QR Code Dialog',
      (tester) async {
        await DialogUtils.pumpDialog(
          tester,
          (context) => QrCodeDialog.show(
            context,
            title: 'Test Title',
            data: 'Test Data',
          ),
        );
        await TestUtils.preCacheWalletLogoForQrImageView(tester);
        await screenMatchesGolden('qr_code_dialog');
      },
    );
  });
}
