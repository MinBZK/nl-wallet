part of 'qr_scan_bloc.dart';

abstract class QrScanState extends Equatable {
  const QrScanState();
}

class QrScanInitial extends QrScanState {
  @override
  List<Object> get props => [];
}

class QrScanScanning extends QrScanState {
  @override
  List<Object> get props => [];
}

class QrScanSuccess extends QrScanState {
  final QrRequest request;

  const QrScanSuccess(this.request);

  @override
  List<Object> get props => [request];
}

class QrScanFailure extends QrScanState {
  @override
  List<Object> get props => [];
}

class QrScanNoPermission extends QrScanState {
  final bool permanentlyDenied;

  const QrScanNoPermission(this.permanentlyDenied);

  @override
  List<Object> get props => [permanentlyDenied];
}
