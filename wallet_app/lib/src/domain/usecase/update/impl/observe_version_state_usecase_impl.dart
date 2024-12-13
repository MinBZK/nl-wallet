import 'package:rxdart/rxdart.dart';

import '../../../../data/repository/version/version_state_repository.dart';
import '../../../../wallet_constants.dart';
import '../observe_version_state_usecase.dart';

class ObserveVersionStateUsecaseImpl extends ObserveVersionStateUsecase {
  final VersionStateRepository _versionStateRepository;

  ObserveVersionStateUsecaseImpl(this._versionStateRepository);

  /// The debounce is added to make sure that all depending components stay in sync.
  @override
  Stream<VersionState> invoke() =>
      _versionStateRepository.observeVersionState().debounceTime(kDefaultAnimationDuration);
}
