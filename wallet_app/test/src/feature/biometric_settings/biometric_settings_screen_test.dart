import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/usecase/biometrics/biometrics.dart';
import 'package:wallet/src/feature/biometric_settings/biometric_settings_screen.dart';
import 'package:wallet/src/feature/biometric_settings/bloc/biometric_settings_bloc.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';

class MockBiometricSettingsBloc extends MockBloc<BiometricSettingsEvent, BiometricSettingsState>
    implements BiometricSettingsBloc {
  @override
  Biometrics supportedBiometrics = Biometrics.none;
}

void main() {
  group('goldens', () {
    testGoldens('Biometrics face loaded light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const BiometricSettingScreen().withState<BiometricSettingsBloc, BiometricSettingsState>(
              MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.face,
              const BiometricSettingsLoaded(biometricLoginEnabled: true),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'face_loaded.light');
    });

    testGoldens('Biometrics finger loaded light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const BiometricSettingScreen().withState<BiometricSettingsBloc, BiometricSettingsState>(
              MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.fingerprint,
              const BiometricSettingsLoaded(biometricLoginEnabled: false),
            ),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'finger_loaded.dark');
    });

    testGoldens('Biometrics finger loaded light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const BiometricSettingScreen().withState<BiometricSettingsBloc, BiometricSettingsState>(
              MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.fingerprint,
              const BiometricSettingsLoading(),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'finger_loading.light');
    });

    testGoldens('Biometrics some loaded dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const BiometricSettingScreen().withState<BiometricSettingsBloc, BiometricSettingsState>(
              MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.some,
              const BiometricSettingsLoaded(biometricLoginEnabled: true),
            ),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'some_loaded.dark');
    });

    testGoldens('Biometrics setup required light', (tester) async {
      final bloc = MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.some;

      whenListen(
        bloc,
        Stream<BiometricSettingsState>.value(const BiometricSettingsSetupRequired()),
        initialState: const BiometricSettingsLoaded(biometricLoginEnabled: false),
      );

      await tester.pumpWidgetWithAppWrapper(
        const BiometricSettingScreen(),
        providers: [RepositoryProvider<BiometricSettingsBloc>(create: (c) => bloc)],
      );

      await screenMatchesGolden(tester, 'setup_required.light');
    });
  });
}
