import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/bloc/error_state.dart';
import '../../../../domain/model/result/application_error.dart';
import '../../../../domain/model/result/result.dart';
import '../../../../domain/usecase/engagement/start_qr_engagement_usecase.dart';

part 'qr_present_event.dart';
part 'qr_present_state.dart';

class QrPresentBloc extends Bloc<QrPresentEvent, QrPresentState> {
  // ignore: unused_field
  final StartQrEngagementUseCase _startQrEngagementUseCase;

  QrPresentBloc(this._startQrEngagementUseCase) : super(const QrPresentInitial()) {
    on<QrPresentStartRequested>(_onStartRequested);
    on<QrPresentStopRequested>((event, emit) {});
    on<QrPresentEventReceived>((event, emit) {});
  }

  FutureOr<void> _onStartRequested(QrPresentStartRequested event, Emitter<QrPresentState> emit) async {
    emit(const QrPresentInitial());

    // final result = await _startQrEngagementUseCase.invoke();
    final result = const Result.success('qr_content'); // TODO(Rob): Replace with start when stop is ready
    await result.process(
      onSuccess: (qrContents) => emit(QrPresentServerStarted(qrContents)),
      onError: (error) => emit(QrPresentError(error)),
    );
  }

  @override
  Future<dynamic> close() {
    if (state is QrPresentConnected) {
      // Do not kill BLE server, we will navigate to [DisclosureScreen] who will now manage the BLE connection.
    } else {
      // TODO(Rob): Stop BLE server
    }
    return super.close();
  }
}
