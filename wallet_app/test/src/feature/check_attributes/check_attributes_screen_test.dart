import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/check_attributes/bloc/check_attributes_bloc.dart';
import 'package:wallet/src/feature/check_attributes/check_attributes_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../util/device_utils.dart';

class MockCheckAttributesBloc extends MockBloc<CheckAttributesEvent, CheckAttributesState>
    implements CheckAttributesBloc {}

void main() {
  group('goldens', () {
    DeviceBuilder deviceBuilder(WidgetTester tester) {
      return DeviceUtils.deviceBuilderWithPrimaryScrollController
        ..addScenario(
          widget: CheckAttributesScreen(
            onDataIncorrectPressed: () {},
          ).withState<CheckAttributesBloc, CheckAttributesState>(
            MockCheckAttributesBloc(),
            CheckAttributesSuccess(
              card: WalletMockData.card,
              attributes: WalletMockData.card.attributes,
            ),
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
          onDataIncorrectPressed: () => isCalled = true,
        ).withState<CheckAttributesBloc, CheckAttributesState>(
          MockCheckAttributesBloc(),
          CheckAttributesSuccess(
            card: WalletMockData.card,
            attributes: WalletMockData.card.attributes,
          ),
        ),
      );

      await tester.tap(find.textContaining('Something not right'));
      expect(isCalled, isTrue);
    });
  });
}
