import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:vibration/vibration.dart';

part 'scan_qr_event.dart';
part 'scan_qr_state.dart';

class ScanQrBloc extends Bloc<ScanQrEvent, ScanQrState> {
  ScanQrBloc() : super(ScanQrInitial()) {
    on<ScanQrCheckPermission>(_onCheckPermission);
    on<ScanQrCodeDetected>(_onQrCodeDetected);
    on<ScanQrReset>(_onReset);
    add(const ScanQrCheckPermission());
  }

  void _onCheckPermission(ScanQrCheckPermission event, emit) async {
    final status = await Permission.camera.request();
    if (status.isGranted) {
      emit(ScanQrScanning());
    } else {
      emit(ScanQrNoPermission(status.isPermanentlyDenied));
    }
  }

  void _onQrCodeDetected(ScanQrCodeDetected event, emit) {
    final barcode = event.code;
    Vibration.vibrate();
    if (barcode.rawValue == null) {
      emit(ScanQrError());
    } else {
      emit(ScanQrSuccess(barcode.rawValue!));
    }
  }

  void _onReset(ScanQrReset event, emit) {
    emit(ScanQrInitial());
    add(const ScanQrCheckPermission());
  }
}
