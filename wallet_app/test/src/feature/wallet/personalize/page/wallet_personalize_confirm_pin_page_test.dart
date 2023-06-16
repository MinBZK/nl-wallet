import 'dart:ui';

import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/wallet/personalize/page/wallet_personalize_confirm_pin_page.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mocks.dart';
import '../../../../util/device_utils.dart';

void main() {
  DeviceBuilder deviceBuilder(WidgetTester tester) {
    return DeviceUtils.accessibilityDeviceBuilder
      ..addScenario(
        widget: WalletPersonalizeConfirmPinPage(
          bloc: PinBloc(Mocks.create()),
          onPinValidated: () {},
        ),
        name: 'error_screen',
      );
  }

  group('Golden Tests', () {
    testGoldens('Accessibility Light Test', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'accessibility_light');
    });

    testGoldens('Accessibility Dark Test', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'accessibility_dark');
    });
  });

  testWidgets('WalletPersonalizeConfirmPinPage renders the correct title & subtitle', (tester) async {
    final locale = await AppLocalizations.delegate.load(const Locale('en'));
    await tester.pumpWidget(
      WalletAppTestWidget(
        child: WalletPersonalizeConfirmPinPage(
          onPinValidated: () {},
          bloc: PinBloc(Mocks.create()),
        ),
      ),
    );

    // Setup finders
    final titleFinder = find.text(locale.walletPersonalizeConfirmPinPageTitle);
    final descriptionFinder = find.text(locale.walletPersonalizeConfirmPinPageDescription);

    // Verify all expected widgets show up once
    expect(titleFinder, findsOneWidget);
    expect(descriptionFinder, findsOneWidget);
  });
}
