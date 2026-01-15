import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/biometrics/get_available_biometrics_usecase.dart';
import 'package:wallet/src/util/helper/onboarding_helper.dart';

import '../../mocks/wallet_mocks.dart';

void main() {
  group('OnboardingHelper', () {
    Future<GetAvailableBiometricsUseCase> buildBiometricsUseCase(Biometrics biometricsResult) async {
      // Configure the (mock) usecase
      final getAvailableBiometricsUseCase = MockGetAvailableBiometricsUseCase();
      when(getAvailableBiometricsUseCase.invoke()).thenAnswer((_) async => biometricsResult);
      return getAvailableBiometricsUseCase;
    }

    // Reset value for each test
    setUp(() => OnboardingHelper.initWithValue(0));

    testWidgets('init sets totalSetupSteps to 10 when biometrics available', (tester) async {
      final usecaseFace = await buildBiometricsUseCase(Biometrics.face);
      await OnboardingHelper.init(usecaseFace);
      expect(OnboardingHelper.totalSteps, 10);

      final usecaseFingerprint = await buildBiometricsUseCase(Biometrics.fingerprint);
      await OnboardingHelper.init(usecaseFingerprint);
      expect(OnboardingHelper.totalSteps, 10);

      final usecaseSome = await buildBiometricsUseCase(Biometrics.some);
      await OnboardingHelper.init(usecaseSome);
      expect(OnboardingHelper.totalSteps, 10);
    });

    testWidgets('init sets totalSetupSteps to 9 when biometrics not available', (tester) async {
      final context = await buildBiometricsUseCase(Biometrics.none);
      await OnboardingHelper.init(context);
      expect(OnboardingHelper.totalSteps, 9);
    });

    test('initWithValue sets totalSetupSteps to test value', () {
      // This bypasses platform/Biometrics checks for easier testing
      OnboardingHelper.initWithValue(42);
      expect(OnboardingHelper.totalSteps, 42);
    });
  });
}
