import '../../../../data/repository/close_proximity/close_proximity_repository.dart';
import '../../../model/close_proximity/ble_connection_event.dart';
import '../../wallet_usecase.dart';
import '../observe_close_proximity_connection_usecase.dart';

class ObserveCloseProximityConnectionUseCaseImpl extends ObserveCloseProximityConnectionUseCase {
  final CloseProximityRepository _closeProximityRepository;

  ObserveCloseProximityConnectionUseCaseImpl(this._closeProximityRepository);

  @override
  Stream<BleConnectionEvent> invoke() => _closeProximityRepository.observeBleConnectionEvents().handleAppError(
    'Error while observing close proximity connection',
  );
}
