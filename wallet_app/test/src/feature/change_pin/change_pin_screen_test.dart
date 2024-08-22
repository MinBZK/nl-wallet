import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';
import 'package:wallet/src/domain/usecase/pin/check_pin_usecase.dart';
import 'package:wallet/src/feature/change_pin/bloc/change_pin_bloc.dart';
import 'package:wallet/src/feature/change_pin/change_pin_screen.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.mocks.dart';
import '../../util/test_utils.dart';
import '../pin/pin_page_test.dart';

class MockChangePinBloc extends MockBloc<ChangePinEvent, ChangePinState> implements ChangePinBloc {}

void main() {
  group(
    'goldens',
    () {
      testGoldens(
        'ChangePinInitial',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen()
                .withState<ChangePinBloc, ChangePinState>(
                  MockChangePinBloc(),
                  const ChangePinInitial(),
                )
                .withState<PinBloc, PinState>(
                  MockPinBloc(),
                  const PinEntryInProgress(0),
                ),
            providers: [RepositoryProvider<CheckPinUseCase>(create: (c) => MockCheckPinUseCase())],
          );
          await tester.pumpAndSettle();
          await screenMatchesGolden(tester, 'change_pin/initial.light');
        },
      );

      testGoldens(
        'ChangePinInitial Dark',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen()
                .withState<ChangePinBloc, ChangePinState>(
                  MockChangePinBloc(),
                  const ChangePinInitial(),
                )
                .withState<PinBloc, PinState>(
                  MockPinBloc(),
                  const PinEntryInProgress(0),
                ),
            providers: [RepositoryProvider<CheckPinUseCase>(create: (c) => MockCheckPinUseCase())],
            brightness: Brightness.dark,
          );
          await tester.pumpAndSettle();
          await screenMatchesGolden(tester, 'change_pin/initial.dark');
        },
      );

      testGoldens(
        'ChangePinSelectNewPinInProgress',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              const ChangePinSelectNewPinInProgress(3),
            ),
          );
          await tester.pumpAndSettle();
          await screenMatchesGolden(tester, 'change_pin/select_new_pin.light');
        },
      );

      testGoldens(
        'ChangePinSelectNewPinInProgress - landscape',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              const ChangePinSelectNewPinInProgress(3),
            ),
            surfaceSize: const Size(812, 375),
          );
          await tester.pumpAndSettle();
          await screenMatchesGolden(tester, 'change_pin/select_new_pin.landscape.light');
        },
      );

      testGoldens(
        'ChangePinSelectNewPinFailed - sequential',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              const ChangePinSelectNewPinFailed(reason: PinValidationError.sequentialDigits),
            ),
          );
          await tester.pumpAndSettle();
          await screenMatchesGolden(tester, 'change_pin/select_new_pin_failed.light');
        },
      );

      testGoldens(
        'ChangePinConfirmNewPinInProgress',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              const ChangePinConfirmNewPinInProgress(3),
            ),
          );
          await tester.pumpAndSettle();
          await screenMatchesGolden(tester, 'change_pin/confirm_new_pin.light');
        },
      );

      testGoldens(
        'ChangePinConfirmNewPinFailed - retry allowed',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              const ChangePinConfirmNewPinFailed(retryAllowed: true),
            ),
          );
          await tester.pumpAndSettle();
          await screenMatchesGolden(tester, 'change_pin/confirm_new_pin_failed.light');
        },
      );

      testGoldens(
        'ChangePinConfirmNewPinFailed - retry not allowed',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              const ChangePinConfirmNewPinFailed(retryAllowed: false),
            ),
          );
          await tester.pumpAndSettle();
          await screenMatchesGolden(tester, 'change_pin/confirm_new_pin_failed_no_retry.light');
        },
      );

      testGoldens(
        'ChangePinUpdating',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              ChangePinUpdating(),
            ),
          );
          await tester.pumpAndSettle();
          await screenMatchesGolden(tester, 'change_pin/updating.light');
        },
      );

      testGoldens(
        'ChangePinGenericError',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              const ChangePinGenericError(error: 'generic'),
            ),
          );
          await tester.pumpAndSettle();
          await screenMatchesGolden(tester, 'change_pin/generic_error.light');
        },
      );

      testGoldens(
        'ChangePinNetworkError',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              const ChangePinNetworkError(hasInternet: true, error: 'network'),
            ),
          );
          await tester.pumpAndSettle();
          await screenMatchesGolden(tester, 'change_pin/network_error.light');
        },
      );

      testGoldens(
        'ChangePinCompleted',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              ChangePinCompleted(),
            ),
          );
          await tester.pumpAndSettle();
          await screenMatchesGolden(tester, 'change_pin/completed.light');
        },
      );

      testGoldens(
        'ChangePinCompleted - dark',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              ChangePinCompleted(),
            ),
            brightness: Brightness.dark,
          );
          await tester.pumpAndSettle();
          await screenMatchesGolden(tester, 'change_pin/completed.dark');
        },
      );
    },
  );

  group(
    'widgets',
    () {
      testWidgets(
        'verify ChangePinInitial state shows correct title',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen()
                .withState<ChangePinBloc, ChangePinState>(
                  MockChangePinBloc(),
                  const ChangePinInitial(),
                )
                .withState<PinBloc, PinState>(
                  MockPinBloc(),
                  const PinEntryInProgress(0),
                ),
            providers: [RepositoryProvider<CheckPinUseCase>(create: (c) => MockCheckPinUseCase())],
          );
          await tester.pumpAndSettle();

          final l10n = await TestUtils.englishLocalizations;
          final titleFinder = find.text(l10n.changePinScreenEnterCurrentPinTitle);
          expect(titleFinder, findsOneWidget);
        },
      );

      testWidgets(
        'verify ChangePinSelectNewPinInProgress shows correct title',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              const ChangePinSelectNewPinInProgress(3),
            ),
          );
          await tester.pumpAndSettle();

          final l10n = await TestUtils.englishLocalizations;
          final titleFinder = find.text(l10n.changePinScreenSelectNewPinTitle);
          expect(titleFinder, findsOneWidget);
        },
      );

      testWidgets(
        'verify ChangePinConfirmNewPinInProgress shows correct title',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              const ChangePinConfirmNewPinInProgress(3),
            ),
          );
          await tester.pumpAndSettle();

          final l10n = await TestUtils.englishLocalizations;
          final titleFinder = find.text(l10n.changePinScreenConfirmNewPinTitle);
          expect(titleFinder, findsOneWidget);
        },
      );

      testWidgets(
        'verify ChangePinUpdating shows correct title & description',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              ChangePinUpdating(),
            ),
          );
          await tester.pumpAndSettle();

          final l10n = await TestUtils.englishLocalizations;
          final titleFinder = find.text(l10n.changePinScreenUpdatingTitle);
          final descriptionFinder = find.text(l10n.changePinScreenUpdatingDescription);
          expect(titleFinder, findsOneWidget);
          expect(descriptionFinder, findsOneWidget);
        },
      );

      testWidgets(
        'verify ChangePinCompleted shows correct title',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              ChangePinCompleted(),
            ),
          );
          await tester.pumpAndSettle();

          final l10n = await TestUtils.englishLocalizations;
          final titleFinder = find.text(l10n.changePinScreenSuccessTitle);
          final descriptionFinder = find.text(l10n.changePinScreenSuccessDescription);
          expect(titleFinder, findsOneWidget);
          expect(descriptionFinder, findsOneWidget);
        },
      );
    },
  );
}
