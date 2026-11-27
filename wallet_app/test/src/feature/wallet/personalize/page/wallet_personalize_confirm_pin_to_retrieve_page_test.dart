import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/pin/pin_page.dart';
import 'package:wallet/src/feature/wallet/personalize/page/wallet_personalize_confirm_pin_to_retrieve_page.dart';
import 'package:wallet/src/feature/wallet/personalize/wallet_personalize_setup_failed_screen.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';
import '../../../../test_util/test_utils.dart';

class _MockPinBloc extends MockBloc<PinEvent, PinState> implements PinBloc {}

void main() {
  group('widgets', () {
    testWidgets('renders the correct title inside a PinPage', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        WalletPersonalizeConfirmPinToRetrievePage(
          bloc: PinBloc(Mocks.create()),
          onPidRetrieved: (_) {},
          onRetrievePidFailed: (_, _) {},
        ),
      );

      expect(find.byType(PinPage), findsOneWidget);
      expect(find.byKey(const Key('personalizeConfirmPinToRetrievePage')), findsOneWidget);
      expect(find.text(l10n.walletPersonalizeConfirmPinPageTitle, findRichText: true), findsOneWidget);
    });

    testWidgets('invokes onPidRetrieved with the attributes when the pin is validated', (tester) async {
      final attributes = [WalletMockData.textDataAttribute];
      final bloc = _MockPinBloc();
      whenListen(
        bloc,
        Stream.fromIterable([PinValidateSuccess(result: attributes)]),
        initialState: const PinEntryInProgress(0),
      );

      List<Attribute>? retrieved;
      await tester.pumpWidgetWithAppWrapper(
        WalletPersonalizeConfirmPinToRetrievePage(
          bloc: bloc,
          onPidRetrieved: (result) => retrieved = result,
          onRetrievePidFailed: (_, _) => fail('onRetrievePidFailed should not be called'),
        ),
      );

      expect(retrieved, attributes);
    });

    testWidgets('invokes onRetrievePidFailed when the bloc emits an error state', (tester) async {
      final bloc = _MockPinBloc();
      const error = PinValidateError(error: GenericError('error', sourceError: 'test'));
      whenListen(bloc, Stream.fromIterable([error]), initialState: const PinEntryInProgress(0));

      PinState? failedState;
      await tester.pumpWidgetWithAppWrapper(
        WalletPersonalizeConfirmPinToRetrievePage(
          bloc: bloc,
          onPidRetrieved: (_) => fail('onPidRetrieved should not be called'),
          onRetrievePidFailed: (_, state) => failedState = state as PinState,
        ),
      );

      expect(failedState, error);
    });

    testWidgets('shows the setup failed screen when the pin is blocked during setup', (tester) async {
      final bloc = _MockPinBloc();
      whenListen(
        bloc,
        Stream.fromIterable([const PinValidateBlocked()]),
        initialState: const PinEntryInProgress(0),
      );

      await tester.pumpWidgetWithAppWrapper(
        WalletPersonalizeConfirmPinToRetrievePage(
          bloc: bloc,
          onPidRetrieved: (_) => fail('onPidRetrieved should not be called'),
          onRetrievePidFailed: (_, _) => fail('onRetrievePidFailed should not be called'),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(WalletPersonalizeSetupFailedScreen), findsOneWidget);
    });
  });
}
