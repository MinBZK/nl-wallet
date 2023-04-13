part of 'flashlight_cubit.dart';

abstract class FlashlightState extends Equatable {
  const FlashlightState();
}

class FlashlightInitial extends FlashlightState {
  @override
  List<Object> get props => [];
}

class FlashlightToggled extends FlashlightState {
  const FlashlightToggled();

  @override
  List<Object> get props => [];
}

class FlashlightSuccess extends FlashlightState {
  final bool on;

  const FlashlightSuccess(this.on);

  @override
  List<Object> get props => [on];
}
