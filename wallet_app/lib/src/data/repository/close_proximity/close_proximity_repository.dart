import '../../../domain/model/close_proximity/ble_connection_event.dart';

abstract class CloseProximityRepository {
  Future<bool> getWrprcStrictMode();

  Future<void> setWrprcStrictMode({required bool enabled});

  Future<String> startCloseProximityDisclosure();

  Stream<BleConnectionEvent> observeBleConnectionEvents();
}
