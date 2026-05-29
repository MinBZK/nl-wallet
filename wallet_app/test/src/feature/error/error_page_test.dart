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
      await screenMatchesGolden('page/generic_retry');
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
      await screenMatchesGolden('page/generic_close');
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
      await screenMatchesGolden('page/network');
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
      await screenMatchesGolden('page/no_internet');
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
      await screenMatchesGolden('page/session_expired');
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
      await screenMatchesGolden('page/cancelled_session');
    });

    testGoldens('ErrorPage.relyingParty', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => ErrorPage.relyingParty(
            context,
            organizationName: 'Verifier X',
            onPrimaryActionPressed: () {},
            style: ErrorCtaStyle.retry,
          ),
        ),
      );
      await screenMatchesGolden('page/relying_party');
    });

    testGoldens('ErrorPage.relyingParty (for issuance)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => ErrorPage.relyingParty(
            context,
            organizationName: 'Issuer Y',
            onPrimaryActionPressed: () {},
            style: ErrorCtaStyle.close,
            useIssuanceStyle: true,
          ),
        ),
      );
      await screenMatchesGolden('page/relying_party.issuance');
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
      await screenMatchesGolden('page/from_error_no_internet');
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
      await screenMatchesGolden('page/generic_retry_dark_landscape');
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

  group('fromError', () {
    testWidgets('maps GenericError to ErrorPage.generic', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            final page = ErrorPage.fromError(
              context,
              const GenericError('msg', sourceError: 'error'),
              onPrimaryActionPressed: () {},
              style: ErrorCtaStyle.retry,
            );
            final expected = ErrorPage.generic(context, style: ErrorCtaStyle.retry);
            expect(page.title, expected.title);
            expect(page.description, expected.description);
            expect(page.illustration, expected.illustration);
            return const SizedBox.shrink();
          },
        ),
      );
    });

    testWidgets('maps NetworkError(hasInternet: true) to ErrorPage.server', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            final page = ErrorPage.fromError(
              context,
              const NetworkError(hasInternet: true, sourceError: 'error'),
              onPrimaryActionPressed: () {},
              style: ErrorCtaStyle.retry,
            );
            final expected = ErrorPage.server(context, style: ErrorCtaStyle.retry);
            expect(page.title, expected.title);
            expect(page.description, expected.description);
            return const SizedBox.shrink();
          },
        ),
      );
    });

    testWidgets('maps NetworkError(hasInternet: false) to ErrorPage.noInternet', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            final page = ErrorPage.fromError(
              context,
              const NetworkError(hasInternet: false, sourceError: 'error'),
              onPrimaryActionPressed: () {},
              style: ErrorCtaStyle.retry,
            );
            final expected = ErrorPage.noInternet(context, style: ErrorCtaStyle.retry);
            expect(page.title, expected.title);
            expect(page.description, expected.description);
            return const SizedBox.shrink();
          },
        ),
      );
    });

    testWidgets('maps SessionError(state: expired) to ErrorPage.sessionExpired', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            final page = ErrorPage.fromError(
              context,
              const SessionError(state: SessionState.expired, sourceError: 'error'),
              onPrimaryActionPressed: () {},
              style: ErrorCtaStyle.retry,
            );
            final expected = ErrorPage.sessionExpired(context, style: ErrorCtaStyle.retry);
            expect(page.title, expected.title);
            expect(page.description, expected.description);
            return const SizedBox.shrink();
          },
        ),
      );
    });

    testWidgets('maps SessionError(state: cancelled) to ErrorPage.cancelledSession', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            final page = ErrorPage.fromError(
              context,
              const SessionError(state: SessionState.cancelled, sourceError: 'error'),
              onPrimaryActionPressed: () {},
              style: ErrorCtaStyle.retry,
              organizationName: 'Org',
            );
            final expected = ErrorPage.cancelledSession(
              context,
              style: ErrorCtaStyle.retry,
              organizationName: 'Org',
            );
            expect(page.title, expected.title);
            expect(page.description, expected.description);
            return const SizedBox.shrink();
          },
        ),
      );
    });

    testWidgets('maps RelyingPartyError to ErrorPage.relyingParty', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            final page = ErrorPage.fromError(
              context,
              const RelyingPartyError(sourceError: 'error'),
              onPrimaryActionPressed: () {},
              style: ErrorCtaStyle.retry,
            );
            final expected = ErrorPage.relyingParty(context, style: ErrorCtaStyle.retry);
            expect(page.title, expected.title);
            expect(page.description, expected.description);
            return const SizedBox.shrink();
          },
        ),
      );
    });

    testWidgets('maps RelyingPartyError with organizationName to ErrorPage.relyingParty', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            const orgName = 'Test Organization';
            final error = RelyingPartyError(
              sourceError: 'error',
              organizationName: {const Locale('en'): orgName},
            );
            final page = ErrorPage.fromError(
              context,
              error,
              onPrimaryActionPressed: () {},
              style: ErrorCtaStyle.retry,
            );
            final expected = ErrorPage.relyingParty(
              context,
              style: ErrorCtaStyle.retry,
              organizationName: orgName,
            );
            expect(page.title, expected.title);
            expect(page.description, expected.description);
            return const SizedBox.shrink();
          },
        ),
      );
    });

    testWidgets('maps HardwareUnsupportedError to ErrorPage.deviceIncompatible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            final page = ErrorPage.fromError(
              context,
              const HardwareUnsupportedError(sourceError: 'error'),
              onPrimaryActionPressed: () {},
              style: ErrorCtaStyle.retry,
            );
            final expected = ErrorPage.deviceIncompatible(context);
            expect(page.title, expected.title);
            expect(page.description, expected.description);
            return const SizedBox.shrink();
          },
        ),
      );
    });

    testWidgets('maps unknown error to ErrorPage.generic (default case)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            final page = ErrorPage.fromError(
              context,
              const ExternalScannerError(sourceError: 'error'),
              onPrimaryActionPressed: () {},
              style: ErrorCtaStyle.retry,
            );
            final expected = ErrorPage.generic(context, style: ErrorCtaStyle.retry);
            expect(page.title, expected.title);
            return const SizedBox.shrink();
          },
        ),
      );
    });
  });
}
