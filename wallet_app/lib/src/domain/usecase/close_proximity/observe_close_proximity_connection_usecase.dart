import '../../model/close_proximity/ble_connection_event.dart';
import '../wallet_usecase.dart';

abstract class ObserveCloseProximityConnectionUseCase extends WalletUseCase {
  Stream<BleConnectionEvent> invoke();
}
