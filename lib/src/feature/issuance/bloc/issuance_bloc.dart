import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

part 'issuance_event.dart';
part 'issuance_state.dart';

class IssuanceBloc extends Bloc<IssuanceEvent, IssuanceState> {
  IssuanceBloc() : super(IssuanceInitial()) {
    on<IssuanceEvent>((event, emit) {
      // TODO: implement event handler
    });
  }
}
