import 'package:wallet_core/core.dart' as core;

import '../../../domain/model/close_proximity/ble_connection_event.dart';
import '../../../wallet_core/error/core_error.dart';
import '../mapper.dart';

class CloseProximityDisclosureUpdateMapper
    extends Mapper<core.CloseProximityDisclosureFlutterUpdate, BleConnectionEvent> {
  final Mapper<String, CoreError> _errorMapper;

  CloseProximityDisclosureUpdateMapper(this._errorMapper);

  @override
  BleConnectionEvent map(core.CloseProximityDisclosureFlutterUpdate input) {
    return switch (input) {
      core.CloseProximityDisclosureFlutterUpdate_Connecting() => const BleConnecting(),
      core.CloseProximityDisclosureFlutterUpdate_Connected() => const BleConnected(),
      core.CloseProximityDisclosureFlutterUpdate_DeviceRequestReceived() => const BleDeviceRequestReceived(),
      core.CloseProximityDisclosureFlutterUpdate_Disconnected() => const BleDisconnected(),
      core.CloseProximityDisclosureFlutterUpdate_Errored(:final error) => BleError(_errorMapper.map(error)),
    };
  }
}
