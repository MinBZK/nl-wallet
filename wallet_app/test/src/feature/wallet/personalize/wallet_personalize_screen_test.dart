import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/data_attribute.dart';
import 'package:wallet/src/domain/usecase/pin/confirm_transaction_usecase.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/wallet/personalize/bloc/wallet_personalize_bloc.dart';
import 'package:wallet/src/feature/wallet/personalize/wallet_personalize_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/mock_data.dart';
import '../../../util/device_utils.dart';
import '../../../util/test_utils.dart';
import '../../pin/pin_page_test.dart';

class MockWalletPersonalizeBloc extends MockBloc<WalletPersonalizeEvent, WalletPersonalizeState>
    implements WalletPersonalizeBloc {}

class MockConfirmTransactionUseCase implements ConfirmTransactionUseCase {
  @override
  Future<CheckPinResult> invoke(String pin) => throw UnimplementedError();
}

void main() {
  const kPidId = 'id';

  /// All attribtues here are needed to satisfy the [PidAttributeMapper] used when rendering the [WalletPersonalizeCheckData] state.
  const pidAttributes = [
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Voornamen',
      value: 'John',
      type: AttributeType.firstNames,
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Achternaam',
      value: 'Doe',
      type: AttributeType.lastName,
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Geboortenaam',
      value: 'John',
      type: AttributeType.birthName,
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Geslacht',
      value: 'Male',
      type: AttributeType.gender,
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Geboortedatum',
      value: '01-01-2000',
      type: AttributeType.birthDate,
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Geboorteplaats',
      value: 'Amsterdam',
      type: AttributeType.birthPlace,
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Geboorteland',
      value: 'Nederland',
      type: AttributeType.birthCountry,
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Burgerservicenummer (BSN)',
      value: '******999',
      type: AttributeType.citizenshipNumber,
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Nationaliteit',
      value: 'Nederlands',
      type: AttributeType.nationality,
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Stad',
      value: 'Amsterdam',
      type: AttributeType.city,
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Postcode',
      value: '1234AB',
      type: AttributeType.postalCode,
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Straatnaam',
      value: 'Dorpsstraat',
      type: AttributeType.streetName,
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Huisnummer',
      value: '1A',
      type: AttributeType.houseNumber,
      sourceCardId: kPidId,
    ),
  ];

  group('goldens', () {
    ///FIXME: Also verify WalletPersonalizeConnectDigid ?

    testGoldens('WalletPersonalizeInitial Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              const WalletPersonalizeInitial(),
            ),
            name: 'initial',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/initial.light');
    });

    testGoldens('WalletPersonalizeLoadInProgress Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              const WalletPersonalizeLoadInProgress(5),
            ),
            name: 'load_in_progress',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/load_in_progress.light');
    });

    testGoldens('WalletPersonalizeAuthenticating Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              const WalletPersonalizeAuthenticating(),
            ),
            name: 'authenticating',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/authenticating.light');
    });

    testGoldens('WalletPersonalizeConfirmPin Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: RepositoryProvider<ConfirmTransactionUseCase>.value(
              value: MockConfirmTransactionUseCase(),
              child: const WalletPersonalizeScreen()
                  .withState<WalletPersonalizeBloc, WalletPersonalizeState>(
                    MockWalletPersonalizeBloc(),
                    const WalletPersonalizeConfirmPin(),
                  )
                  .withState<PinBloc, PinState>(
                    MockPinBloc(),
                    const PinEntryInProgress(0),
                  ),
            ),
            name: 'confirm_pin',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/confirm_pin.light');
    });

    testGoldens('WalletPersonalizeCheckData Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              const WalletPersonalizeCheckData(availableAttributes: pidAttributes),
            ),
            name: 'check_data',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/check_data.light');
    });

    testGoldens('WalletPersonalizeSuccess Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              const WalletPersonalizeSuccess([WalletMockData.card, WalletMockData.altCard]),
            ),
            name: 'success',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/success.light');
    });

    testGoldens('WalletPersonalizeSuccess Dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              const WalletPersonalizeSuccess([WalletMockData.card, WalletMockData.altCard]),
            ),
            name: 'success',
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/success.dark');
    });

    testGoldens('WalletPersonalizeFailure Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              WalletPersonalizeFailure(),
            ),
            name: 'failure',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/failure.light');
    });

    testGoldens('WalletPersonalizeDigidFailure Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              WalletPersonalizeDigidFailure(),
            ),
            name: 'digid_failure',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/digid_failure.light');
    });
  });

  group('widgets', () {
    testWidgets('continue button is shown on the success page', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeSuccess([WalletMockData.card]),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.walletPersonalizeSuccessPageContinueCta), findsOneWidget);
    });
  });
}
