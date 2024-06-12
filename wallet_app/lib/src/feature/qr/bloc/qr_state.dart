part of 'qr_bloc.dart';

sealed class QrState extends Equatable {
  const QrState();
}

class QrScanInitial extends QrState {
  @override
  List<Object> get props => [];
}

class QrScanScanning extends QrState {
  @override
  List<Object> get props => [];
}

class QrScanLoading extends QrState {
  const QrScanLoading();

  @override
  List<Object> get props => [];
}

class QrScanSuccess extends QrState {
  final NavigationRequest request;

  const QrScanSuccess(this.request);

  @override
  List<Object> get props => [request];
}

class QrScanFailure extends QrState {
  @override
  List<Object> get props => [];
}

class QrScanNoPermission extends QrState {
  final bool permanentlyDenied;

  const QrScanNoPermission({required this.permanentlyDenied});

  @override
  List<Object> get props => [permanentlyDenied];
}
