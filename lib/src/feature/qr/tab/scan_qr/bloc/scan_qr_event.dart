part of 'scan_qr_bloc.dart';

abstract class ScanQrEvent extends Equatable {
  const ScanQrEvent();
}

class ScanQrCodeDetected extends ScanQrEvent {
  final Barcode code;

  const ScanQrCodeDetected(this.code);

  @override
  List<Object?> get props => [code];
}

class ScanQrReset extends ScanQrEvent {
  const ScanQrReset();

  @override
  List<Object?> get props => [];
}

class ScanQrCheckPermission extends ScanQrEvent {
  const ScanQrCheckPermission();

  @override
  List<Object?> get props => [];
}
