abstract class RequestBiometricsUsecase {
  Future<RequestBiometricsResult> invoke();
}

enum RequestBiometricsResult { success, failure, setupRequired }
