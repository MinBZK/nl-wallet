import 'package:flutter/cupertino.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/pin_timeout/widget/pin_timeout_description.dart';

import '../../../../wallet_app_test_widget.dart';

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
    final AppLocalizations locale = await AppLocalizations.delegate.load(const Locale('en'));
    final leadingTextFinder = find.textContaining(locale.pinTimeoutScreenTimeoutPrefix, findRichText: true);
    final timeLeftFinder = find.textContaining(locale.generalMinutes(2), findRichText: true);

    // Verify all expected widgets show up once
    expect(leadingTextFinder, findsOneWidget);
    expect(timeLeftFinder, findsOneWidget);
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
    final AppLocalizations locale = await AppLocalizations.delegate.load(const Locale('en'));
    final leadingTextFinder = find.textContaining(locale.pinTimeoutScreenTimeoutPrefix, findRichText: true);
    final timeLeftFinder = find.textContaining(locale.generalSeconds(30), findRichText: true);

    // Verify all expected widgets show up once
    expect(leadingTextFinder, findsOneWidget);
    expect(timeLeftFinder, findsOneWidget);
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
