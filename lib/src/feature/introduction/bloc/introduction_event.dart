part of 'introduction_bloc.dart';

abstract class IntroductionEvent extends Equatable {
  const IntroductionEvent();

  @override
  List<Object?> get props => [];
}

class IntroductionNextPressed extends IntroductionEvent {
  const IntroductionNextPressed();
}

class IntroductionBackPressed extends IntroductionEvent {
  const IntroductionBackPressed();
}
