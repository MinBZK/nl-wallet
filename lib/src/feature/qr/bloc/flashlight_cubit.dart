import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

part 'flashlight_state.dart';

class FlashlightCubit extends Cubit<FlashlightState> {
  FlashlightCubit() : super(FlashlightInitial());

  void confirmState(bool on) {
    emit(FlashlightSuccess(on));
  }

  void toggle() {
    emit(const FlashlightToggled());
  }
}
