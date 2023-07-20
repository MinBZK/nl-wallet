import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/check_attributes/check_attributes_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/mock_data.dart';
import '../../util/device_utils.dart';

void main() {
  group('goldens', () {
    DeviceBuilder deviceBuilder(WidgetTester tester) {
      return DeviceUtils.deviceBuilderWithPrimaryScrollController
        ..addScenario(
          widget: CheckAttributesScreen(
            cardsToAttributes: {
              WalletMockData.card: WalletMockData.card.attributes,
              WalletMockData.altCard: WalletMockData.altCard.attributes.sublist(1, 2),
            },
            onDataIncorrectPressed: () {},
          ),
        );
    }

    testGoldens('check attributes light', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'light');
    });

    testGoldens('check attributes dark', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'dark');
    });
  });

  group('widgets', () {
    testWidgets('onDataIncorrect is triggered when pressed', (tester) async {
      bool isCalled = false;
      await tester.pumpWidgetWithAppWrapper(
        CheckAttributesScreen(
          cardsToAttributes: {WalletMockData.card: WalletMockData.card.attributes},
          onDataIncorrectPressed: () => isCalled = true,
        ),
      );

      await tester.tap(find.textContaining('Something not right'));
      expect(isCalled, isTrue);
    });
  });
}
