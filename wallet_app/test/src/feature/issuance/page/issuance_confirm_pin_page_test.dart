import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:wallet/src/domain/model/issuance/disclose_for_issuance_result.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/issuance/page/issuance_confirm_pin_for_disclosure_page.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/pin/pin_page.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

class _MockPinBloc extends MockBloc<PinEvent, PinState> implements PinBloc {}

void main() {
  group('goldens', () {
    testGoldens('Confirm page light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        IssuanceConfirmPinForDisclosurePage(
          bloc: PinBloc(Mocks.create()),
          onPinValidated: (_) {},
          onConfirmWithPinFailed: (_, _) {},
        ),
      );
      await screenMatchesGolden('issuance_confirm_pin/light');
    });

    testGoldens('Confirm page dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        IssuanceConfirmPinForDisclosurePage(
          bloc: PinBloc(Mocks.create()),
          onPinValidated: (_) {},
          onConfirmWithPinFailed: (_, _) {},
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('issuance_confirm_pin/dark');
    });
  });

  group('widgets', () {
    testWidgets('IssuanceConfirmPinPage renders the correct title & subtitle', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        IssuanceConfirmPinForDisclosurePage(
          bloc: PinBloc(Mocks.create()),
          onPinValidated: (_) {},
          onConfirmWithPinFailed: (_, _) {},
        ),
      );

      // Setup finders
      final titleFinder = find.text(l10n.issuanceConfirmPinPageTitle, findRichText: true);

      // Verify all expected widgets show up once
      expect(titleFinder, findsOneWidget);
    });

    testWidgets('renders a custom title when provided', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        IssuanceConfirmPinForDisclosurePage(
          bloc: PinBloc(Mocks.create()),
          title: 'Custom title',
          onPinValidated: (_) {},
          onConfirmWithPinFailed: (_, _) {},
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(find.byType(PinPage), findsOneWidget);
      expect(find.text('Custom title', findRichText: true), findsOneWidget);
      expect(find.text(l10n.issuanceConfirmPinPageTitle, findRichText: true), findsNothing);
    });

    testWidgets('invokes onPinValidated with the DiscloseForIssuanceResult when the pin is validated', (tester) async {
      final expected = DiscloseForIssuanceResult.previews([WalletMockData.card]);
      final bloc = _MockPinBloc();
      whenListen(
        bloc,
        Stream.fromIterable([PinValidateSuccess(result: expected)]),
        initialState: const PinEntryInProgress(0),
      );

      DiscloseForIssuanceResult? result;
      await tester.pumpWidgetWithAppWrapper(
        IssuanceConfirmPinForDisclosurePage(
          bloc: bloc,
          onPinValidated: (it) => result = it,
          onConfirmWithPinFailed: (_, _) => fail('onConfirmWithPinFailed should not be called'),
        ),
      );

      expect(result, expected);
    });

    testWidgets('invokes onConfirmWithPinFailed when the bloc emits an error state', (tester) async {
      final bloc = _MockPinBloc();
      const error = PinValidateError(error: GenericError('error', sourceError: 'test'));
      whenListen(bloc, Stream.fromIterable([error]), initialState: const PinEntryInProgress(0));

      PinState? failedState;
      await tester.pumpWidgetWithAppWrapper(
        IssuanceConfirmPinForDisclosurePage(
          bloc: bloc,
          onPinValidated: (_) => fail('onPinValidated should not be called'),
          onConfirmWithPinFailed: (_, state) => failedState = state as PinState,
        ),
      );

      expect(failedState, error);
    });
  });
}
