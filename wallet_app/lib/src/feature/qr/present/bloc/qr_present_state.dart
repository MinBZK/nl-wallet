part of 'qr_present_bloc.dart';

/// Base state for the QR code presentation flow.
sealed class QrPresentState extends Equatable {
  const QrPresentState();

  @override
  List<Object> get props => [];
}

/// Initial state before the QR presentation flow starts.
class QrPresentInitial extends QrPresentState {
  const QrPresentInitial();
}

/// The BLE server started; [qrContents] is ready to be displayed as a QR code.
class QrPresentServerStarted extends QrPresentState {
  final String qrContents;

  const QrPresentServerStarted(this.qrContents);

  @override
  List<Object> get props => [...super.props, qrContents];
}

/// A remote device is initiating a connection.
class QrPresentConnecting extends QrPresentState {
  const QrPresentConnecting();
}

/// A remote device is connected; [deviceRequestReceived] signals readiness to navigate.
class QrPresentConnected extends QrPresentState {
  final bool deviceRequestReceived;

  const QrPresentConnected({required this.deviceRequestReceived});

  @override
  List<Object> get props => [...super.props, deviceRequestReceived];
}

/// The connection attempt with the remote device failed.
class QrPresentConnectionFailed extends QrPresentState {
  const QrPresentConnectionFailed();
}

/// An error occurred during the QR presentation flow.
class QrPresentError extends QrPresentState implements ErrorState {
  @override
  final ApplicationError error;

  const QrPresentError(this.error);

  @override
  List<Object> get props => [...super.props, error];
}
