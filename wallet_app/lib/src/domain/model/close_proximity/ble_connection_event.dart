import 'package:equatable/equatable.dart';

import '../../../wallet_core/error/core_error.dart';

/// Base class for Bluetooth Low Energy (BLE) connection events.
sealed class BleConnectionEvent extends Equatable {
  const BleConnectionEvent();

  @override
  List<Object?> get props => [];
}

/// BLE advertising started; [engagement] is ready to be shared (e.g., via QR code).
class BleAdvertising extends BleConnectionEvent {
  final String engagement;

  const BleAdvertising(this.engagement);

  @override
  List<Object?> get props => [...super.props, engagement];
}

/// A remote device is connecting.
class BleConnecting extends BleConnectionEvent {
  const BleConnecting();
}

/// A remote device has connected.
class BleConnected extends BleConnectionEvent {
  const BleConnected();
}

/// The remote device has transmitted a DeviceRequest.
class BleDeviceRequestReceived extends BleConnectionEvent {
  const BleDeviceRequestReceived();
}

/// The remote device has disconnected.
class BleDisconnected extends BleConnectionEvent {
  const BleDisconnected();
}

/// An error occurred during the BLE operation.
class BleError extends BleConnectionEvent {
  final CoreError error;

  const BleError(this.error);

  @override
  List<Object?> get props => [...super.props, error];
}
