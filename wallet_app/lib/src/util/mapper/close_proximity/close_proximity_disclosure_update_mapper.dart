import 'package:wallet_core/core.dart' as core;

import '../../../domain/model/close_proximity/ble_connection_event.dart';
import '../mapper.dart';

class CloseProximityDisclosureUpdateMapper
    extends Mapper<core.CloseProximityDisclosureFlutterUpdate, BleConnectionEvent> {
  CloseProximityDisclosureUpdateMapper();

  @override
  BleConnectionEvent map(core.CloseProximityDisclosureFlutterUpdate input) {
    return switch (input) {
      core.CloseProximityDisclosureFlutterUpdate.Connecting => const BleConnecting(),
      core.CloseProximityDisclosureFlutterUpdate.Connected => const BleConnected(),
      core.CloseProximityDisclosureFlutterUpdate.DeviceRequestReceived => const BleDeviceRequestReceived(),
      core.CloseProximityDisclosureFlutterUpdate.Disconnected => const BleDisconnected(),
    };
  }
}
