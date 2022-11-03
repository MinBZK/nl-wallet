import 'package:flutter_bloc/flutter_bloc.dart';

class BackButtonVisibilityCubit extends Cubit<bool> {
  BackButtonVisibilityCubit() : super(false);

  void showBackButton(bool show) => emit(show);
}
