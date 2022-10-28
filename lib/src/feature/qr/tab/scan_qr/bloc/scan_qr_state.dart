part of 'scan_qr_bloc.dart';

abstract class ScanQrState extends Equatable {
  const ScanQrState();
}

class ScanQrInitial extends ScanQrState {
  @override
  List<Object> get props => [];
}

class ScanQrScanning extends ScanQrState {
  @override
  List<Object> get props => [];
}

class ScanQrSuccess extends ScanQrState {
  final String data;

  const ScanQrSuccess(this.data);

  @override
  List<Object> get props => [data];
}

class ScanQrError extends ScanQrState {
  @override
  List<Object> get props => [];
}

class ScanQrNoPermission extends ScanQrState {
  final bool permanentlyDenied;

  const ScanQrNoPermission(this.permanentlyDenied);

  @override
  List<Object> get props => [permanentlyDenied];
}
