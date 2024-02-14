part of 'qr_bloc.dart';

abstract class QrEvent extends Equatable {
  const QrEvent();
}

class QrScanCodeDetected extends QrEvent {
  final Barcode code;

  const QrScanCodeDetected(this.code);

  @override
  List<Object?> get props => [code];
}

class QrScanReset extends QrEvent {
  const QrScanReset();

  @override
  List<Object?> get props => [];
}

class QrScanCheckPermission extends QrEvent {
  const QrScanCheckPermission();

  @override
  List<Object?> get props => [];
}
