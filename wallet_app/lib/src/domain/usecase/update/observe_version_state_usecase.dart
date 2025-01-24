import '../../model/update/version_state.dart';

export '../../model/update/version_state.dart';

abstract class ObserveVersionStateUsecase {
  Stream<VersionState> invoke();
}
