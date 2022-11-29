part of 'introduction_bloc.dart';

abstract class IntroductionState extends Equatable {
  int get totalSteps => 4;

  int get currentStep => 0;

  bool get canGoBack => false;

  bool get didGoBack => false;

  const IntroductionState();

  @override
  List<Object?> get props => [canGoBack, didGoBack];
}

class IntroductionAppDisclaimer extends IntroductionState {
  final bool afterBackPressed;

  const IntroductionAppDisclaimer({this.afterBackPressed = false});

  @override
  int get currentStep => 0;

  @override
  bool get canGoBack => false;

  @override
  bool get didGoBack => afterBackPressed;
}

class IntroductionAppIntroduction extends IntroductionState {
  final bool afterBackPressed;

  const IntroductionAppIntroduction({this.afterBackPressed = false});

  @override
  int get currentStep => 1;

  @override
  bool get canGoBack => true;

  @override
  bool get didGoBack => afterBackPressed;
}

class IntroductionAppBenefits extends IntroductionState {
  final bool afterBackPressed;

  const IntroductionAppBenefits({this.afterBackPressed = false});

  @override
  int get currentStep => 2;

  @override
  bool get canGoBack => true;

  @override
  bool get didGoBack => afterBackPressed;
}

class IntroductionAppSecurity extends IntroductionState {
  @override
  int get currentStep => 3;

  @override
  bool get canGoBack => true;
}
