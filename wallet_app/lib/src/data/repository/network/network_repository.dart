export '../../../domain/model/disclosure/start_disclosure_result.dart';

abstract class NetworkRepository {
  /// Check if the device has an active internet connection
  Future<bool> hasInternet();
}
