import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:vibration/vibration.dart';

import '../../../../environment.dart';
import '../../../domain/model/navigation/navigation_request.dart';
import '../../../domain/usecase/permission/check_has_permission_usecase.dart';
import '../../../domain/usecase/qr/decode_qr_usecase.dart';

part 'qr_event.dart';
part 'qr_state.dart';

/// We deliberately delay the QR processing so the user has a moment to realize the QR has been scanned successfully.
final kProcessingDelay = Duration(milliseconds: Environment.isTest ? 25 : 500);

class QrBloc extends Bloc<QrEvent, QrState> {
  final DecodeQrUseCase _decodeQrUseCase;
  final CheckHasPermissionUseCase _checkHasPermissionUseCase;

  QrBloc(this._decodeQrUseCase, this._checkHasPermissionUseCase) : super(QrScanInitial()) {
    on<QrScanCheckPermission>(_onCheckPermission);
    on<QrScanCodeDetected>(_onCodeDetected);
    on<QrScanReset>(_onReset);
  }

  Future<void> _onCheckPermission(QrScanCheckPermission event, emit) async {
    final result = await _checkHasPermissionUseCase.invoke(Permission.camera);
    if (result.isGranted) {
      emit(QrScanScanning());
    } else {
      emit(QrScanNoPermission(permanentlyDenied: result.isPermanentlyDenied));
    }
  }

  Future<void> _onCodeDetected(QrScanCodeDetected event, emit) async {
    if (state is QrScanLoading || state is QrScanSuccess || state is QrScanFailure) {
      return; //Already processing a QR code
    }
    emit(const QrScanLoading());
    unawaited(Vibration.vibrate());
    try {
      final request = await _decodeQrUseCase.invoke(event.code);
      await Future.delayed(kProcessingDelay);
      emit(QrScanSuccess(request!));
    } catch (ex) {
      Fimber.e('Failed to decode QR code', ex: ex);
      emit(QrScanFailure());
    }
  }

  void _onReset(QrScanReset event, emit) {
    emit(QrScanInitial());
    add(const QrScanCheckPermission());
  }
}
