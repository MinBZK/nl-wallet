import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

class WalletAppBlocObserver extends BlocObserver {
  @override
  void onChange(BlocBase<dynamic> bloc, Change<dynamic> change) {
    super.onChange(bloc, change);
    Fimber.d('> ${bloc.runtimeType} $change');
  }
}
