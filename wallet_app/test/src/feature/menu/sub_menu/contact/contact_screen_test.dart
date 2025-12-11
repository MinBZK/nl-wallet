import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';
import 'package:wallet/src/domain/usecase/wallet/observe_wallet_locked_usecase.dart';
import 'package:wallet/src/feature/menu/sub_menu/contact/contact_screen.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mocks.dart';
import '../../../../test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('ltc26 ContactScreen light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ContactScreen(),
        providers: [
          Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
        ],
      );
      await screenMatchesGolden('light');
    });

    testGoldens('ltc26 ContactScreen dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ContactScreen(),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [
          Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
        ],
      );
      await screenMatchesGolden('dark.landscape');
    });
  });
}
