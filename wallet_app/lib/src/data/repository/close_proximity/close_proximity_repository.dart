import '../../../domain/model/close_proximity/ble_connection_event.dart';

abstract class CloseProximityRepository {
  Future<String> startCloseProximityDisclosure();

  Stream<BleConnectionEvent> observeBleConnectionEvents();
}
