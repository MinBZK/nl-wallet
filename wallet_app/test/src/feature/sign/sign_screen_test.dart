import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/usecase/sign/accept_sign_agreement_usecase.dart';
import 'package:wallet/src/feature/common/widget/centered_loading_indicator.dart';
import 'package:wallet/src/feature/organization/approve/organization_approve_page.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/sign/bloc/sign_bloc.dart';
import 'package:wallet/src/feature/sign/page/check_agreement_page.dart';
import 'package:wallet/src/feature/sign/page/confirm_agreement_page.dart';
import 'package:wallet/src/feature/sign/page/sign_confirm_pin_page.dart';
import 'package:wallet/src/feature/sign/page/sign_generic_error_page.dart';
import 'package:wallet/src/feature/sign/page/sign_stopped_page.dart';
import 'package:wallet/src/feature/sign/page/sign_success_page.dart';
import 'package:wallet/src/feature/sign/sign_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../mocks/wallet_mocks.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';
import '../pin/pin_page_test.dart';

class MockSignBloc extends MockBloc<SignEvent, SignState> implements SignBloc {}

void main() {
  group('goldens', () {
    testGoldens('SignInitial Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          const SignInitial(),
        ),
      );
      await screenMatchesGolden('initial.light');
    });

    testGoldens('SignLoadInProgress Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          const SignLoadInProgress(),
        ),
      );
      await screenMatchesGolden('load_in_progress.light');
    });

    testGoldens('SignCheckOrganization Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          SignCheckOrganization(relyingParty: WalletMockData.organization),
        ),
      );
      await screenMatchesGolden('check_organization.light');
    });

    testGoldens('SignCheckAgreement Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          SignCheckAgreement(
            relyingParty: WalletMockData.organization,
            trustProvider: WalletMockData.organization,
            document: WalletMockData.document,
          ),
        ),
      );
      await screenMatchesGolden('check_agreement.light');
    });

    testGoldens('SignConfirmPin Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        RepositoryProvider<AcceptSignAgreementUseCase>.value(
          value: MockAcceptSignAgreementUseCase(),
          child: const SignScreen()
              .withState<SignBloc, SignState>(
                MockSignBloc(),
                const SignConfirmPin(),
              )
              .withState<PinBloc, PinState>(
                MockPinBloc(),
                const PinEntryInProgress(0),
              ),
        ),
      );
      await screenMatchesGolden('provide_pin.light');
    });

    testGoldens('SignConfirmAgreement Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          SignConfirmAgreement(
            relyingParty: WalletMockData.organization,
            document: WalletMockData.document,
            trustProvider: WalletMockData.organization,
            policy: WalletMockData.policy,
            requestedAttributes: [WalletMockData.textDataAttribute],
          ),
        ),
      );
      await screenMatchesGolden('confirm_agreement.light');
    });

    testGoldens('SignConfirmAgreement Dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          SignConfirmAgreement(
            relyingParty: WalletMockData.organization,
            requestedAttributes: [WalletMockData.textDataAttribute],
            policy: WalletMockData.policy,
            trustProvider: WalletMockData.organization,
            document: WalletMockData.document,
          ),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('confirm_agreement.dark');
    });

    testGoldens('SignSuccess Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          SignSuccess(relyingParty: WalletMockData.organization),
        ),
      );
      await screenMatchesGolden('success.light');
    });

    testGoldens('SignError Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          const SignError(),
        ),
      );
      await screenMatchesGolden('sign_error.light');
    });

    testGoldens('SignStopped Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          const SignStopped(),
        ),
      );
      await screenMatchesGolden('stopped.light');
    });
  });

  group('widgets', () {
    testWidgets('continue cta is visible when Sign is completed', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          SignSuccess(relyingParty: WalletMockData.organization),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.signSuccessPageCloseCta), findsOneWidget);
    });

    testWidgets('SignSuccess renders the SignSuccessPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          SignSuccess(relyingParty: WalletMockData.organization),
        ),
      );
      expect(find.byType(SignSuccessPage), findsOneWidget);
    });

    testWidgets('SignInitial state renders loader', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          const SignInitial(),
        ),
      );
      expect(find.byType(CenteredLoadingIndicator), findsOneWidget);
    });

    testWidgets('SignLoadInProgress state renders loader', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          const SignLoadInProgress(),
        ),
      );
      expect(find.byType(CenteredLoadingIndicator), findsOneWidget);
    });

    testWidgets('SignCheckOrganization state renders OrganizationApprovePage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          SignCheckOrganization(relyingParty: WalletMockData.organization),
        ),
      );
      expect(find.byType(OrganizationApprovePage), findsOneWidget);
    });

    testWidgets('SignCheckAgreement state renders CheckAgreementPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          SignCheckAgreement(
            relyingParty: WalletMockData.organization,
            trustProvider: WalletMockData.organization,
            document: WalletMockData.document,
          ),
        ),
      );
      expect(find.byType(CheckAgreementPage), findsOneWidget);
    });

    testWidgets('SignConfirmAgreement state renders ConfirmAgreementPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          SignConfirmAgreement(
            relyingParty: WalletMockData.organization,
            trustProvider: WalletMockData.organization,
            document: WalletMockData.document,
            policy: WalletMockData.policy,
            requestedAttributes: const [],
          ),
        ),
      );
      expect(find.byType(ConfirmAgreementPage), findsOneWidget);
    });

    testWidgets('SignConfirmPin state renders SignConfirmPinPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          const SignConfirmPin(),
        ),
        providers: [
          RepositoryProvider<PinBloc>(create: (_) => MockPinBloc()),
          RepositoryProvider<AcceptSignAgreementUseCase>(create: (_) => MockAcceptSignAgreementUseCase()),
        ],
      );
      expect(find.byType(SignConfirmPinPage), findsOneWidget);
    });

    testWidgets('SignError state renders SignGenericErrorPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          const SignError(),
        ),
      );
      expect(find.byType(SignGenericErrorPage), findsOneWidget);
    });

    testWidgets('SignStopped state renders SignStoppedPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          const SignStopped(),
        ),
      );
      expect(find.byType(SignStoppedPage), findsOneWidget);
    });
  });
}
