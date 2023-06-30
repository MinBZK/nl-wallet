import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

part 'home_event.dart';
part 'home_state.dart';

class HomeBloc extends Bloc<HomeEvent, HomeState> {
  HomeBloc() : super(const HomeScreenSelect(HomeTab.cards)) {
    on<HomeTabPressed>(_onHomeTabPressedEvent);
  }

  void _onHomeTabPressedEvent(HomeTabPressed event, emit) {
    emit(HomeScreenSelect(
      event.tab,
      uid: _generateUid(event.forceStateRefresh),
    ));
  }

  /// Generates a (naive) UID if [forceStateRefresh] is true
  int? _generateUid(bool forceStateRefresh) {
    if (forceStateRefresh) return DateTime.now().millisecondsSinceEpoch;
    return null;
  }
}
