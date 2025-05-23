import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/issuance/page/issuance_relying_party_error_page.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('Organization unknown - light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(IssuanceRelyingPartyErrorPage(onClosePressed: () {}));
      await screenMatchesGolden('issuance_rp_error/organization_unknown');
    });

    testGoldens('Organization Known - dark ', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        IssuanceRelyingPartyErrorPage(
          onClosePressed: () {},
          organizationName: 'Organization X',
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('issuance_rp_error/organization_known.dark');
    });
  });
}
