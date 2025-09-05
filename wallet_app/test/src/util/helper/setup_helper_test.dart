import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/biometrics/get_available_biometrics_usecase.dart';
import 'package:wallet/src/util/helper/setup_helper.dart';

import '../../mocks/wallet_mocks.dart';

void main() {
  group('SetupHelper', () {
    // Helper to build a fake Bloc context for tests
    Future<BuildContext> buildContextWithBiometrics(
      WidgetTester tester,
      Biometrics biometricsResult,
    ) async {
      // Configure the (mock) usecase
      final getAvailableBiometricsUseCase = MockGetAvailableBiometricsUseCase();
      when(getAvailableBiometricsUseCase.invoke()).thenAnswer((_) async => biometricsResult);

      // Create a context which inherits the usecase
      late BuildContext context;
      final widget = RepositoryProvider<GetAvailableBiometricsUseCase>.value(
        value: getAvailableBiometricsUseCase,
        child: Builder(
          builder: (c) {
            context = c;
            return const SizedBox.shrink();
          },
        ),
      );
      await tester.pumpWidget(widget);

      // Return the context in question
      return context;
    }

    // Reset value for each test
    setUp(() => SetupHelper.initWithValue(0));

    testWidgets('init sets totalSetupSteps to 9 when biometrics available', (tester) async {
      final context = await buildContextWithBiometrics(tester, Biometrics.face);
      await SetupHelper.init(context);
      expect(SetupHelper.totalSetupSteps, 9);

      final context2 = await buildContextWithBiometrics(tester, Biometrics.fingerprint);
      await SetupHelper.init(context2);
      expect(SetupHelper.totalSetupSteps, 9);

      final context3 = await buildContextWithBiometrics(tester, Biometrics.some);
      await SetupHelper.init(context3);
      expect(SetupHelper.totalSetupSteps, 9);
    });

    testWidgets('init sets totalSetupSteps to 8 when biometrics not available', (tester) async {
      final context = await buildContextWithBiometrics(tester, Biometrics.none);
      await SetupHelper.init(context);
      expect(SetupHelper.totalSetupSteps, 8);
    });

    test('initWithValue sets totalSetupSteps to test value', () {
      // This bypasses platform/Biometrics checks for easier testing
      SetupHelper.initWithValue(42);
      expect(SetupHelper.totalSetupSteps, 42);
    });
  });
}
