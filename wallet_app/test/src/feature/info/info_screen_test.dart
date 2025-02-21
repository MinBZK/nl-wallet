import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/info/info_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';

void main() {
  group('widgets', () {
    testWidgets('provided title and description are shown', (WidgetTester tester) async {
      await tester.pumpWidgetWithAppWrapper(const InfoScreen(title: 'title', description: 'description'));

      expect(find.text('title'), findsAtLeast(1));
      expect(find.text('description'), findsOneWidget);
    });
  });

  group('goldens', () {
    DeviceBuilder deviceBuilder(WidgetTester tester) => DeviceUtils.deviceBuilderWithPrimaryScrollController;

    testGoldens('InfoScreen', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester)
          ..addScenario(
            widget: InfoScreen(
              title: l10n.detailsIncorrectScreenTitle,
              description: l10n.detailsIncorrectScreenDescription,
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'info_screen');
    });
  });
}
