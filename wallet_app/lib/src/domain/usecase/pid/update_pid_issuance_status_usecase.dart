import '../../../../bridge_generated.dart';

abstract class UpdatePidIssuanceStatusUseCase {
  Future<void> invoke(PidIssuanceEvent state);
}
