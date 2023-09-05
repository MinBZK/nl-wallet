import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:fimber/fimber.dart';
import 'package:internet_connection_checker/internet_connection_checker.dart';

import '../check_has_internet_usecase.dart';

class CheckHasInternetUseCaseImpl implements CheckHasInternetUseCase {
  final Connectivity connectivity;
  final InternetConnectionChecker connectionChecker;

  CheckHasInternetUseCaseImpl(this.connectivity, this.connectionChecker);

  @override
  Future<bool> invoke() async {
    try {
      final result = await connectivity.checkConnectivity();
      if (result == ConnectivityResult.none) return false;
      return await connectionChecker.hasConnection;
    } catch (ex, stack) {
      Fimber.e('Failed to check connectivity', ex: ex, stacktrace: stack);
      return false;
    }
  }
}
