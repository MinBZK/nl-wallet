import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/common/widget/button/primary_button.dart';
import 'package:wallet/src/feature/error/error_page.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('ErrorPage.generic - retry', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => ErrorPage.generic(
            context,
            onPrimaryActionPressed: () {},
            style: ErrorCtaStyle.retry,
          ),
        ),
      );
      await screenMatchesGolden('error/generic_retry');
    });

    testGoldens('ErrorPage.generic - close', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => ErrorPage.generic(
            context,
            onPrimaryActionPressed: () {},
            style: ErrorCtaStyle.close,
          ),
        ),
      );
      await screenMatchesGolden('error/generic_close');
    });

    testGoldens('ErrorPage.network', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => ErrorPage.server(
            context,
            onPrimaryActionPressed: () {},
            style: ErrorCtaStyle.retry,
          ),
        ),
      );
      await screenMatchesGolden('error/network');
    });

    testGoldens('ErrorPage.noInternet', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => ErrorPage.noInternet(
            context,
            onPrimaryActionPressed: () {},
            style: ErrorCtaStyle.retry,
          ),
        ),
      );
      await screenMatchesGolden('error/no_internet');
    });

    testGoldens('ErrorPage.sessionExpired', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => ErrorPage.sessionExpired(
            context,
            onPrimaryActionPressed: () {},
            style: ErrorCtaStyle.retry,
          ),
        ),
      );
      await screenMatchesGolden('error/session_expired');
    });

    testGoldens('ErrorPage.cancelledSession', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => ErrorPage.cancelledSession(
            context,
            organizationName: 'Test Org',
            onPrimaryActionPressed: () {},
          ),
        ),
      );
      await screenMatchesGolden('error/cancelled_session');
    });

    testGoldens('ErrorPage.relyingParty', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => ErrorPage.relyingParty(
            context,
            organizationName: 'Test Org',
            onPrimaryActionPressed: () {},
            style: ErrorCtaStyle.retry,
          ),
        ),
      );
      await screenMatchesGolden('error/relying_party');
    });

    testGoldens('ErrorPage.fromError - NetworkError (no internet)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => ErrorPage.fromError(
            context,
            const NetworkError(hasInternet: false, sourceError: 'error'),
            onPrimaryActionPressed: () {},
            style: ErrorCtaStyle.retry,
          ),
        ),
      );
      await screenMatchesGolden('error/from_error_no_internet');
    });

    testGoldens('ErrorPage - dark mode landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => ErrorPage.generic(
            context,
            onPrimaryActionPressed: () {},
            style: ErrorCtaStyle.retry,
          ),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('error/generic_retry_dark_landscape');
    });
  });

  group('interactions', () {
    testWidgets('primary action button triggers callback', (tester) async {
      bool pressed = false;
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => ErrorPage.generic(
            context,
            onPrimaryActionPressed: () => pressed = true,
            style: ErrorCtaStyle.retry,
          ),
        ),
      );

      await tester.tap(find.byType(PrimaryButton));
      expect(pressed, isTrue);
    });
  });
}
