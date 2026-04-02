part of 'qr_present_bloc.dart';

/// Base class for all events related to the QR presentation feature.
abstract class QrPresentEvent extends Equatable {
  const QrPresentEvent();

  @override
  List<Object?> get props => [];
}

/// Event triggered when the QR presentation process is requested to start.
class QrPresentStartRequested extends QrPresentEvent {
  const QrPresentStartRequested();
}

/// Event triggered when the required permissions (e.g., Bluetooth) are denied.
/// This typically happens when the screen is opened but the necessary permissions
/// are not available.
class QrPresentPermissionDenied extends QrPresentEvent {
  const QrPresentPermissionDenied();
}

/// Event triggered when the QR presentation process is requested to stop.
class QrPresentStopRequested extends QrPresentEvent {
  const QrPresentStopRequested();
}

/// Event triggered when a [BleConnectionEvent] is received during the
/// QR presentation process.
class QrPresentEventReceived extends QrPresentEvent {
  final BleConnectionEvent event;

  const QrPresentEventReceived(this.event);

  @override
  List<Object?> get props => [...super.props, event];
}
