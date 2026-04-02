import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/bloc/error_state.dart';
import '../../../../domain/model/close_proximity/ble_connection_event.dart';
import '../../../../domain/model/result/application_error.dart';
import '../../../../domain/usecase/close_proximity/observe_close_proximity_connection_usecase.dart';
import '../../../../domain/usecase/close_proximity/start_close_proximity_disclosure_usecase.dart';
import '../../../../domain/usecase/disclosure/cancel_disclosure_usecase.dart';

part 'qr_present_event.dart';

part 'qr_present_state.dart';

class QrPresentBloc extends Bloc<QrPresentEvent, QrPresentState> {
  final StartCloseProximityDisclosureUseCase _startCloseProximityDisclosureUseCase;
  final ObserveCloseProximityConnectionUseCase _observeCloseProximityConnectionUseCase;
  final CancelDisclosureUseCase _cancelDisclosureUseCase;

  StreamSubscription? _connectionSubscription;

  QrPresentBloc(
    this._startCloseProximityDisclosureUseCase,
    this._observeCloseProximityConnectionUseCase,
    this._cancelDisclosureUseCase,
  ) : super(const QrPresentInitial()) {
    on<QrPresentStartRequested>(_onStartRequested);
    on<QrPresentStopRequested>(_onStopRequested);
    on<QrPresentEventReceived>(_onConnectionEvent);
    on<QrPresentPermissionDenied>(_onPermissionDenied);
  }

  FutureOr<void> _onStartRequested(QrPresentStartRequested event, Emitter<QrPresentState> emit) async {
    emit(const QrPresentInitial());

    final result = await _startCloseProximityDisclosureUseCase.invoke();
    await result.process(
      onSuccess: (qrContents) {
        emit(QrPresentServerStarted(qrContents));
        _startObservingConnection();
      },
      onError: (error) => emit(QrPresentError(error)),
    );
  }

  FutureOr<void> _onStopRequested(QrPresentStopRequested event, Emitter<QrPresentState> emit) async {
    unawaited(_connectionSubscription?.cancel());
    final result = await _cancelDisclosureUseCase.invoke();
    await result.process(
      onSuccess: (_) => emit(const QrPresentConnectionFailed()),
      onError: (error) => emit(QrPresentError(error)),
    );
  }

  void _startObservingConnection() {
    _connectionSubscription?.cancel();
    _connectionSubscription = _observeCloseProximityConnectionUseCase.invoke().listen(
      (event) => add(QrPresentEventReceived(event)),
    );
  }

  FutureOr<void> _onConnectionEvent(QrPresentEventReceived event, Emitter<QrPresentState> emit) {
    switch (event.event) {
      case BleAdvertising():
        Fimber.i('Ble server started');
      case BleConnecting():
        emit(const QrPresentConnecting());
      case BleConnected():
        emit(const QrPresentConnected(deviceRequestReceived: false));
      case BleDeviceRequestReceived():
        emit(const QrPresentConnected(deviceRequestReceived: true));
      case BleDisconnected():
        emit(const QrPresentConnectionFailed());
      case BleError(:final error):
        emit(QrPresentError(error));
    }
  }

  FutureOr<void> _onPermissionDenied(QrPresentPermissionDenied event, Emitter<QrPresentState> emit) {
    unawaited(_connectionSubscription?.cancel());
    unawaited(_cancelDisclosureUseCase.invoke());
    emit(const QrPresentError(GenericError('Bluetooth permissions are missing', sourceError: 'missing_permissions')));
  }

  @override
  Future<dynamic> close() async {
    unawaited(_connectionSubscription?.cancel());
    switch (state) {
      case QrPresentConnected(deviceRequestReceived: true):
        break; // Navigating to [DisclosureScreen], no need to cancel session.
      default:
        await _cancelDisclosureUseCase.invoke();
    }
    return super.close();
  }
}
