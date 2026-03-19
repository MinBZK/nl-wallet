part of 'qr_present_bloc.dart';

sealed class QrPresentState extends Equatable {
  const QrPresentState();

  @override
  List<Object> get props => [];
}

class QrPresentInitial extends QrPresentState {
  const QrPresentInitial();
}

class QrPresentServerStarted extends QrPresentState {
  final String qrContents;

  const QrPresentServerStarted(this.qrContents);

  @override
  List<Object> get props => [...super.props, qrContents];
}

class QrPresentConnecting extends QrPresentState {
  const QrPresentConnecting();
}

class QrPresentConnected extends QrPresentState {
  const QrPresentConnected();
}

class QrPresentConnectionFailed extends QrPresentState {
  const QrPresentConnectionFailed();
}

class QrPresentError extends QrPresentState implements ErrorState {
  @override
  final ApplicationError error;

  const QrPresentError(this.error);

  @override
  List<Object> get props => [...super.props, error];
}
