import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/usecase/biometrics/get_available_biometrics_usecase.dart';

// The amount of steps in the onboarding flow *without* the biometrics step
const _kDefaultSetupSteps = 8;

class SetupHelper {
  SetupHelper._();

  static int? _totalSetupSteps;

  static Future<void> init(BuildContext context) async {
    final biometrics = await context.read<GetAvailableBiometricsUseCase>().invoke();
    switch (biometrics) {
      case Biometrics.face:
      case Biometrics.fingerprint:
      case Biometrics.some:
        _setTotalSetupSteps = _kDefaultSetupSteps + 1;
      case Biometrics.none:
        _setTotalSetupSteps = _kDefaultSetupSteps;
    }
  }

  @visibleForTesting
  static int initWithValue(int totalSteps) => _setTotalSetupSteps = totalSteps;

  // Fetch the total amount of steps the user has to perform during the initial wallet setup.
  // This value is dynamic to make sure it can accommodate the extra 'setup biometrics' step.
  static int get totalSetupSteps {
    if (_totalSetupSteps != null) return _totalSetupSteps!;
    assert(_totalSetupSteps != null, 'totalSetupSteps not initialized, call .init()');
    return _kDefaultSetupSteps;
  }

  static set _setTotalSetupSteps(int value) => _totalSetupSteps = value;
}
