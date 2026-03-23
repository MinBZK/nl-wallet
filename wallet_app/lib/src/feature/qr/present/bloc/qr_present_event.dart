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
  const QrPresentEventReceived();
}
