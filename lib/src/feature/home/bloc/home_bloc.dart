import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

part 'home_event.dart';
part 'home_state.dart';

class HomeBloc extends Bloc<HomeEvent, HomeState> {
  HomeBloc() : super(const HomeScreenSelect(0)) {
    on<HomeTabPressed>(_onHomeTabPressedEvent);
  }

  FutureOr<void> _onHomeTabPressedEvent(HomeTabPressed event, emit) {
    emit(HomeScreenSelect(event.index, timestamp: DateTime.now().millisecondsSinceEpoch));
  }
}
