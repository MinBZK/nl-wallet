import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/cupertino.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:semantic_announcement_tester/semantic_announcement_tester.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/pin/check_pin_usecase.dart';
import 'package:wallet/src/feature/change_pin/bloc/change_pin_bloc.dart';
import 'package:wallet/src/feature/change_pin/change_pin_screen.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/wallet_constants.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.mocks.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';
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
          await screenMatchesGolden('change_pin/initial.light');
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
          await screenMatchesGolden('change_pin/initial.dark');
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
          await screenMatchesGolden('change_pin/select_new_pin.light');
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
          await screenMatchesGolden('change_pin/select_new_pin.landscape.light');
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
          await screenMatchesGolden('change_pin/select_new_pin_failed.light');
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
          await screenMatchesGolden('change_pin/confirm_new_pin.light');
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
          await screenMatchesGolden('change_pin/confirm_new_pin_failed.light');
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
          await screenMatchesGolden('change_pin/confirm_new_pin_failed_no_retry.light');
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
          await screenMatchesGolden('change_pin/updating.light');
        },
      );

      testGoldens(
        'ChangePinGenericError',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              const ChangePinGenericError(error: GenericError('generic', sourceError: 'test')),
            ),
          );
          await tester.pumpAndSettle();
          await screenMatchesGolden('change_pin/generic_error.light');
        },
      );

      testGoldens(
        'ChangePinNetworkError',
        (tester) async {
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen().withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              const ChangePinNetworkError(
                hasInternet: true,
                error: NetworkError(hasInternet: true, sourceError: 'test'),
              ),
            ),
          );
          await tester.pumpAndSettle();
          await screenMatchesGolden('change_pin/network_error.light');
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
          await screenMatchesGolden('change_pin/completed.light');
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
          await screenMatchesGolden('change_pin/completed.dark');
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

      testWidgets(
        'verify announcements of entered digits when selecting new pin',
        (tester) async {
          final mock = MockSemanticAnnouncements(tester);
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen(
              forceAnnouncements: true,
            ).withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              const ChangePinSelectNewPinInProgress(2),
              streamStates: [
                const ChangePinSelectNewPinInProgress(3),
                const ChangePinSelectNewPinInProgress(2, afterBackspacePressed: true),
              ],
            ),
          );
          await tester.pumpAndSettle();

          final l10n = await TestUtils.englishLocalizations;
          expect(
            mock.announcements,
            hasNAnnouncements(
              [
                AnnounceSemanticsEvent(
                  l10n.pinEnteredDigitsAnnouncement(kPinDigits - 3),
                  TextDirection.ltr,
                ),
                AnnounceSemanticsEvent(
                  l10n.pinEnteredDigitsAnnouncement(kPinDigits - 2),
                  TextDirection.ltr,
                ),
              ],
            ),
          );
        },
      );

      testWidgets(
        'verify announcements of entered digits when confirming new pin',
        (tester) async {
          final mock = MockSemanticAnnouncements(tester);
          await tester.pumpWidgetWithAppWrapper(
            const ChangePinScreen(
              forceAnnouncements: true,
            ).withState<ChangePinBloc, ChangePinState>(
              MockChangePinBloc(),
              const ChangePinConfirmNewPinInProgress(1),
              streamStates: [
                const ChangePinConfirmNewPinInProgress(2),
                const ChangePinConfirmNewPinInProgress(1, afterBackspacePressed: true),
                const ChangePinConfirmNewPinInProgress(0),
              ],
            ),
          );
          await tester.pumpAndSettle();

          final l10n = await TestUtils.englishLocalizations;
          expect(
            mock.announcements,
            hasNAnnouncements(
              [
                AnnounceSemanticsEvent(
                  l10n.pinEnteredDigitsAnnouncement(kPinDigits - 2),
                  TextDirection.ltr,
                ),
                AnnounceSemanticsEvent(
                  l10n.pinEnteredDigitsAnnouncement(kPinDigits - 1),
                  TextDirection.ltr,
                ),
                AnnounceSemanticsEvent(
                  l10n.setupSecurityScreenWCAGPinChosenAnnouncement,
                  TextDirection.ltr,
                ),
              ],
            ),
          );
        },
      );
    },
  );
}
