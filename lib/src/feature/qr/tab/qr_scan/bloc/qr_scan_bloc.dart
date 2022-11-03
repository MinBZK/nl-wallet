import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:vibration/vibration.dart';

import '../../../../../domain/model/qr/qr_request.dart';
import '../../../../../domain/usecase/qr/decode_qr_usecase.dart';

part 'qr_scan_event.dart';
part 'qr_scan_state.dart';

class QrScanBloc extends Bloc<QrScanEvent, QrScanState> {
  final DecodeQrUseCase decodeQrUseCase;

  QrScanBloc(this.decodeQrUseCase) : super(QrScanInitial()) {
    on<QrScanCheckPermission>(_onCheckPermission);
    on<QrScanCodeDetected>(onCodeDetected);
    on<QrScanReset>(_onReset);
    add(const QrScanCheckPermission());
  }

  void _onCheckPermission(QrScanCheckPermission event, emit) async {
    final status = await Permission.camera.request();
    if (status.isGranted) {
      emit(QrScanScanning());
    } else {
      emit(QrScanNoPermission(status.isPermanentlyDenied));
    }
  }

  void onCodeDetected(QrScanCodeDetected event, emit) async {
    Vibration.vibrate();
    final request = await decodeQrUseCase.invoke(event.code);
    if (request == null) {
      emit(QrScanFailure());
    } else {
      emit(QrScanSuccess(request));
    }
  }

  void _onReset(QrScanReset event, emit) {
    emit(QrScanInitial());
    add(const QrScanCheckPermission());
  }
}
