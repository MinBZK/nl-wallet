import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/pin_timeout/widget/pin_timeout_description.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../util/test_utils.dart';

void main() {
  testWidgets('verify PinTimeoutDescription renders with 2 minutes left when timeleft >2 mins', (tester) async {
    await tester.pumpWidget(
      WalletAppTestWidget(
        child: PinTimeoutDescription(
          expiryTime: DateTime.now().add(const Duration(seconds: 125)),
          onExpire: () {},
        ),
      ),
    );

    // Setup finders
    final AppLocalizations l10n = await TestUtils.englishLocalizations;
    final timeoutCountDownText = l10n.pinTimeoutScreenTimeoutCountdown(l10n.generalMinutes(2));
    final timeoutCountDownTextFinder = find.text(timeoutCountDownText, findRichText: true);

    // Verify expected widget shows up once
    expect(timeoutCountDownTextFinder, findsOneWidget);
  });

  testWidgets('verify PinTimeoutDescription renders with 30 seconds left', (tester) async {
    await tester.pumpWidget(
      WalletAppTestWidget(
        child: PinTimeoutDescription(
          expiryTime: DateTime.now().add(
            const Duration(
              seconds: 30,
              milliseconds: 750 /* take render time into account */,
            ),
          ),
          onExpire: () {},
        ),
      ),
    );

    // Setup finders
    final AppLocalizations l10n = await TestUtils.englishLocalizations;
    final timeoutCountDownText = l10n.pinTimeoutScreenTimeoutCountdown(l10n.generalSeconds(30));
    final timeoutCountDownTextFinder = find.text(timeoutCountDownText, findRichText: true);

    // Verify expected widget shows up once
    expect(timeoutCountDownTextFinder, findsOneWidget);
  });

  testWidgets('verify onExpire is called when timer expires', (tester) async {
    bool onExpireCalled = false;
    await tester.pumpWidget(
      WalletAppTestWidget(
        child: PinTimeoutDescription(
          expiryTime: DateTime.now().add(const Duration(milliseconds: 200)),
          onExpire: () => onExpireCalled = true,
        ),
      ),
    );

    await tester.pump(const Duration(seconds: 1));

    expect(onExpireCalled, true, reason: 'onExpire should be called after the pump delay');
  });
}
