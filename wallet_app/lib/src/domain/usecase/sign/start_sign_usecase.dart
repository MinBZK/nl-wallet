import '../../model/start_sign_result/start_sign_result.dart';

abstract class StartSignUseCase {
  Future<StartSignResult> invoke(String signUri);
}
