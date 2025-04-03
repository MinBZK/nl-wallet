import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/check_attributes/bloc/check_attributes_bloc.dart';
import 'package:wallet/src/feature/check_attributes/check_attributes_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

class MockCheckAttributesBloc extends MockBloc<CheckAttributesEvent, CheckAttributesState>
    implements CheckAttributesBloc {}

void main() {
  group('goldens', () {
    testGoldens('check attributes light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CheckAttributesScreen(
          onDataIncorrectPressed: () {},
        ).withState<CheckAttributesBloc, CheckAttributesState>(
          MockCheckAttributesBloc(),
          CheckAttributesSuccess(
            card: WalletMockData.card,
            attributes: WalletMockData.card.attributes,
          ),
        ),
      );
      await screenMatchesGolden('light');
    });

    testGoldens('check attributes dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CheckAttributesScreen(
          onDataIncorrectPressed: () {},
        ).withState<CheckAttributesBloc, CheckAttributesState>(
          MockCheckAttributesBloc(),
          CheckAttributesSuccess(
            card: WalletMockData.card,
            attributes: WalletMockData.card.attributes,
          ),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('dark');
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

      final l10n = await TestUtils.englishLocalizations;
      await tester.tap(find.text(l10n.checkAttributesScreenDataIncorrectCta));
      expect(isCalled, isTrue);
    });
  });
}
