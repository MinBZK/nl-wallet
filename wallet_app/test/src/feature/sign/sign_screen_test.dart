import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/trust_provider.dart';
import 'package:wallet/src/domain/usecase/pin/confirm_transaction_usecase.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/sign/bloc/sign_bloc.dart';
import 'package:wallet/src/feature/sign/model/sign_flow.dart';
import 'package:wallet/src/feature/sign/sign_screen.dart';
import 'package:wallet/src/wallet_assets.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/mock_data.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';
import '../pin/pin_page_test.dart';

class MockSignBloc extends MockBloc<SignEvent, SignState> implements SignBloc {}

class MockConfirmTransactionUseCase implements ConfirmTransactionUseCase {
  @override
  Future<CheckPinResult> invoke(String pin) => throw UnimplementedError();
}

void main() {
  const SignFlow mockFlow = SignFlow(
    id: 'id',
    organization: WalletMockData.organization,
    attributes: [WalletMockData.textDataAttribute],
    document: WalletMockData.document,
    policy: WalletMockData.policy,
    trustProvider: TrustProvider(
      name: 'trust provider',
      logoUrl: WalletAssets.logo_sign_provider,
    ),
  );

  group('goldens', () {
    testGoldens('SignInitial Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const SignScreen().withState<SignBloc, SignState>(
              MockSignBloc(),
              const SignInitial(),
            ),
            name: 'initial',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'initial.light');
    });

    testGoldens('SignLoadInProgress Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const SignScreen().withState<SignBloc, SignState>(
              MockSignBloc(),
              const SignLoadInProgress(mockFlow),
            ),
            name: 'load_in_progress',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'load_in_progress.light');
    });

    testGoldens('SignCheckOrganization Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const SignScreen().withState<SignBloc, SignState>(
              MockSignBloc(),
              const SignCheckOrganization(mockFlow),
            ),
            name: 'check_organization',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'check_organization.light');
    });

    testGoldens('SignCheckAgreement Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const SignScreen().withState<SignBloc, SignState>(
              MockSignBloc(),
              const SignCheckAgreement(mockFlow),
            ),
            name: 'check_agreement',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'check_agreement.light');
    });

    testGoldens('SignConfirmPin Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: RepositoryProvider<ConfirmTransactionUseCase>.value(
              value: MockConfirmTransactionUseCase(),
              child: const SignScreen()
                  .withState<SignBloc, SignState>(
                    MockSignBloc(),
                    const SignConfirmPin(mockFlow),
                  )
                  .withState<PinBloc, PinState>(
                    MockPinBloc(),
                    const PinEntryInProgress(0),
                  ),
            ),
            name: 'provide_pin',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'provide_pin.light');
    });

    testGoldens('SignConfirmAgreement Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const SignScreen().withState<SignBloc, SignState>(
              MockSignBloc(),
              const SignConfirmAgreement(mockFlow),
            ),
            name: 'confirm_agreement',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'confirm_agreement.light');
    });

    testGoldens('SignConfirmAgreement Dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const SignScreen().withState<SignBloc, SignState>(
              MockSignBloc(),
              const SignConfirmAgreement(mockFlow),
            ),
            name: 'confirm_agreement',
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'confirm_agreement.dark');
    });

    testGoldens('SignSuccess Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const SignScreen().withState<SignBloc, SignState>(
              MockSignBloc(),
              const SignSuccess(mockFlow),
            ),
            name: 'success',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'success.light');
    });

    testGoldens('SignError Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const SignScreen().withState<SignBloc, SignState>(
              MockSignBloc(),
              const SignError(mockFlow),
            ),
            name: 'sign_error',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'sign_error.light');
    });

    testGoldens('SignStopped Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const SignScreen().withState<SignBloc, SignState>(
              MockSignBloc(),
              const SignStopped(mockFlow),
            ),
            name: 'stopped',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'stopped.light');
    });
  });

  group('widgets', () {
    testWidgets('continue cta is visible when Sign is completed', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SignScreen().withState<SignBloc, SignState>(
          MockSignBloc(),
          const SignSuccess(mockFlow),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.signSuccessPageCloseCta), findsOneWidget);
    });
  });
}
