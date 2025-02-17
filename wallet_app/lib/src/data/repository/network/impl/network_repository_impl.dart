import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:fimber/fimber.dart';
import 'package:internet_connection_checker/internet_connection_checker.dart';

import '../network_repository.dart';

class NetworkRepositoryImpl implements NetworkRepository {
  final Connectivity connectivity;
  final InternetConnectionChecker connectionChecker;

  const NetworkRepositoryImpl(this.connectivity, this.connectionChecker);

  @override
  Future<bool> hasInternet() async {
    try {
      final result = await connectivity.checkConnectivity();
      if (result.firstOrNull == ConnectivityResult.none) return false;
      return await connectionChecker.hasConnection;
    } catch (ex, stack) {
      Fimber.e('Failed to check connectivity', ex: ex, stacktrace: stack);
      return false;
    }
  }
}
