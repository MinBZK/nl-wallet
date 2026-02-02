import 'dart:async';

import 'package:flutter/material.dart';

import '../../domain/usecase/biometrics/get_available_biometrics_usecase.dart';

// The amount of steps in the onboarding flow *without* the biometrics step
const _kDefaultNrOfSteps = 9;

/// A utility class to manage and calculate the total number of steps
/// required in the user onboarding process.
///
/// This helper centralizes the logic for determining the onboarding flow's length,
/// which can vary based on device capabilities (e.g., availability of biometrics).
/// It ensures that the rest of the application can consistently query the total
/// number of steps without having to (asynchronously) resolve the biometrics.
///
/// The class must be initialized once at the start of the app by calling
/// the [init] method. This is crucial for the helper to function correctly.
class OnboardingHelper {
  OnboardingHelper._();

  // Internal static reference to track onboarding steps
  static int? _totalSteps;

  // Initialize the OnboardingHelper, which will resolve the amount of onboarding steps based on the availability of biometrics
  static Future<void> init(GetAvailableBiometricsUseCase usecase) async {
    final biometrics = await usecase.invoke();
    switch (biometrics) {
      case Biometrics.face:
      case Biometrics.fingerprint:
      case Biometrics.some:
        _setTotalSteps = _kDefaultNrOfSteps + 1;
      case Biometrics.none:
        _setTotalSteps = _kDefaultNrOfSteps;
    }
  }

  // Helper function to verify the behaviour of the OnboardingHelper
  @visibleForTesting
  static int initWithValue(int totalSteps) => _setTotalSteps = totalSteps;

  // Fetch the total amount of steps the user has to perform during the initial wallet onboarding.
  // This value is dynamic to make sure it can accommodate the extra 'setup biometrics' step.
  static int get totalSteps {
    if (_totalSteps != null) return _totalSteps!;
    assert(_totalSteps != null, 'totalSteps not initialized, call .init()');
    return _kDefaultNrOfSteps;
  }

  // Internal setter for [_totalSteps], also notifies the completer
  static set _setTotalSteps(int value) => _totalSteps = value;
}
