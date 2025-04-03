import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/usecase/biometrics/biometrics.dart';
import 'package:wallet/src/feature/biometric_settings/biometric_settings_screen.dart';
import 'package:wallet/src/feature/biometric_settings/bloc/biometric_settings_bloc.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';

class MockBiometricSettingsBloc extends MockBloc<BiometricSettingsEvent, BiometricSettingsState>
    implements BiometricSettingsBloc {
  @override
  Biometrics supportedBiometrics = Biometrics.none;
}

void main() {
  group('goldens', () {
    testGoldens('Biometrics face loaded light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const BiometricSettingScreen().withState<BiometricSettingsBloc, BiometricSettingsState>(
          MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.face,
          const BiometricSettingsLoaded(biometricLoginEnabled: true),
        ),
      );
      await screenMatchesGolden('face_loaded.light');
    });

    testGoldens('Biometrics finger loaded light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const BiometricSettingScreen().withState<BiometricSettingsBloc, BiometricSettingsState>(
          MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.fingerprint,
          const BiometricSettingsLoaded(biometricLoginEnabled: false),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('finger_loaded.dark');
    });

    testGoldens('Biometrics finger loaded light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const BiometricSettingScreen().withState<BiometricSettingsBloc, BiometricSettingsState>(
          MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.fingerprint,
          const BiometricSettingsLoading(),
        ),
      );
      await screenMatchesGolden('finger_loading.light');
    });

    testGoldens('Biometrics some loaded dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const BiometricSettingScreen().withState<BiometricSettingsBloc, BiometricSettingsState>(
          MockBiometricSettingsBloc()..supportedBiometrics = Biometrics.some,
          const BiometricSettingsLoaded(biometricLoginEnabled: true),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('some_loaded.dark');
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

      await screenMatchesGolden('setup_required.light');
    });
  });
}
