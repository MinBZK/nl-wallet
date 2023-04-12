part of 'qr_scan_bloc.dart';

abstract class QrScanEvent extends Equatable {
  const QrScanEvent();
}

class QrScanCodeDetected extends QrScanEvent {
  final Barcode code;

  const QrScanCodeDetected(this.code);

  @override
  List<Object?> get props => [code];
}

class QrScanReset extends QrScanEvent {
  const QrScanReset();

  @override
  List<Object?> get props => [];
}

class QrScanCheckPermission extends QrScanEvent {
  const QrScanCheckPermission();

  @override
  List<Object?> get props => [];
}
