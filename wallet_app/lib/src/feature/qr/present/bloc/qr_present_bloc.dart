import 'dart:async';
import 'dart:ui';

import 'package:bluetooth/bluetooth.dart';
import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../data/service/app_lifecycle_service.dart';
import '../../../../domain/model/bloc/error_state.dart';
import '../../../../domain/model/close_proximity/ble_connection_event.dart';
import '../../../../domain/model/result/application_error.dart';
import '../../../../domain/usecase/close_proximity/observe_close_proximity_connection_usecase.dart';
import '../../../../domain/usecase/close_proximity/start_close_proximity_disclosure_usecase.dart';
import '../../../../domain/usecase/disclosure/cancel_disclosure_usecase.dart';
import '../../../../util/extension/core_error_extension.dart';

part 'qr_present_event.dart';

part 'qr_present_state.dart';

class QrPresentBloc extends Bloc<QrPresentEvent, QrPresentState> {
  final StartCloseProximityDisclosureUseCase _startCloseProximityDisclosureUseCase;
  final ObserveCloseProximityConnectionUseCase _observeCloseProximityConnectionUseCase;
  final CancelDisclosureUseCase _cancelDisclosureUseCase;
  final Bluetooth _bluetoothPlugin;
  final AppLifecycleService _lifecycleService;

  StreamSubscription? _connectionSubscription;
  StreamSubscription? _lifecycleSubscription;

  QrPresentBloc(
    this._startCloseProximityDisclosureUseCase,
    this._observeCloseProximityConnectionUseCase,
    this._cancelDisclosureUseCase,
    this._bluetoothPlugin,
    this._lifecycleService,
  ) : super(const QrPresentInitial()) {
    on<QrPresentStartRequested>(_onStartRequested);
    on<QrPresentStopRequested>(_onStopRequested);
    on<QrPresentEventReceived>(_onConnectionEvent);
    on<QrPresentPermissionDenied>(_onPermissionDenied);

    _lifecycleSubscription = _lifecycleService.observe().where((it) => it == .resumed).listen(_onAppResumed);
  }

  FutureOr<void> _onStartRequested(QrPresentStartRequested event, Emitter<QrPresentState> emit) async {
    emit(const QrPresentInitial());

    final bluetoothEnabled = await _bluetoothPlugin.isEnabled();
    if (!bluetoothEnabled) {
      emit(const QrPresentBluetoothDisabled());
      return;
    }

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
    _connectionSubscription = _observeCloseProximityConnectionUseCase
        .invoke()
        .map(QrPresentEventReceived.new)
        .listen(add);
  }

  FutureOr<void> _onConnectionEvent(QrPresentEventReceived event, Emitter<QrPresentState> emit) async {
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
        emit(QrPresentError(await error.asApplicationError()));
    }
  }

  FutureOr<void> _onPermissionDenied(QrPresentPermissionDenied event, Emitter<QrPresentState> emit) {
    unawaited(_connectionSubscription?.cancel());
    unawaited(_cancelDisclosureUseCase.invoke());
    emit(const QrPresentError(GenericError('Bluetooth permissions are missing', sourceError: 'missing_permissions')));
  }

  Future<void> _onAppResumed(AppLifecycleState event) async {
    final bluetoothEnabled = await _bluetoothPlugin.isEnabled();
    switch (state) {
      case QrPresentServerStarted():
        if (!bluetoothEnabled) add(const QrPresentStopRequested());
      case QrPresentBluetoothDisabled():
        if (bluetoothEnabled) add(const QrPresentStartRequested());
      default: // Nothing to do here
    }
  }

  @override
  Future<dynamic> close() async {
    unawaited(_lifecycleSubscription?.cancel());
    unawaited(_connectionSubscription?.cancel());
    Fimber.d('Closing QrPresentBLoC with state: $state');
    switch (state) {
      case QrPresentConnected(deviceRequestReceived: true):
        break; // Navigating to [DisclosureScreen], no need to cancel session.
      default:
        await _cancelDisclosureUseCase.invoke();
    }
    return super.close();
  }
}
