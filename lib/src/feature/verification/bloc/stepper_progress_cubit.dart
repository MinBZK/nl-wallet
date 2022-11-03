import 'package:flutter_bloc/flutter_bloc.dart';

class StepperProgressCubit extends Cubit<double> {
  final int nrOfPages;

  StepperProgressCubit(this.nrOfPages) : super(1 / nrOfPages);

  void setPage(double page) => setProgress((page + 1) / nrOfPages);

  void setProgress(double progress) => emit(progress);
}
