import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

part 'home_event.dart';
part 'home_state.dart';

class HomeBloc extends Bloc<HomeEvent, HomeState> {
  bool _stateToggle = false;

  HomeBloc() : super(const HomeScreenSelect(HomeTab.cards)) {
    on<HomeTabPressed>(_onHomeTabPressedEvent);
  }

  void _onHomeTabPressedEvent(HomeTabPressed event, emit) {
    emit(HomeScreenSelect(
      event.tab,
      stateToggle: _getStateToggle(event.forceStateRefresh),
    ));
  }

  bool _getStateToggle(bool forceStateRefresh) {
    if (forceStateRefresh) {
      _stateToggle = !_stateToggle;
    }
    return _stateToggle;
  }
}
