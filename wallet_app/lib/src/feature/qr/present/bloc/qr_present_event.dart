part of 'qr_present_bloc.dart';

abstract class QrPresentEvent extends Equatable {
  const QrPresentEvent();

  @override
  List<Object?> get props => [];
}

class QrPresentStartRequested extends QrPresentEvent {
  const QrPresentStartRequested();
}

class QrPresentStopRequested extends QrPresentEvent {
  const QrPresentStopRequested();
}

class QrPresentEventReceived extends QrPresentEvent {
  final BleConnectionEvent event;

  const QrPresentEventReceived(this.event);

  @override
  List<Object?> get props => [...super.props, event];
}
