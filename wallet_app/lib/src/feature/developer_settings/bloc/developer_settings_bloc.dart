import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/repository/close_proximity/close_proximity_repository.dart';

part 'developer_settings_event.dart';
part 'developer_settings_state.dart';

class DeveloperSettingsBloc extends Bloc<DeveloperSettingsEvent, DeveloperSettingsState> {
  final CloseProximityRepository _closeProximityRepository;

  DeveloperSettingsBloc(this._closeProximityRepository) : super(const DeveloperSettingsState()) {
    on<DeveloperSettingsLoadTriggered>(_onLoadTriggered);
    on<DeveloperSettingsWrprcStrictModeToggled>(_onWrprcStrictModeToggled);
  }

  Future<void> _onLoadTriggered(
    DeveloperSettingsLoadTriggered event,
    Emitter<DeveloperSettingsState> emit,
  ) async {
    if (state.isBusy) return;
    final previousValue = state.wrprcStrictMode;
    emit(DeveloperSettingsState(wrprcStrictMode: previousValue, isBusy: true));
    try {
      final enabled = await _closeProximityRepository.getWrprcStrictMode();
      emit(DeveloperSettingsState(wrprcStrictMode: enabled));
    } catch (error) {
      Fimber.e('Could not load WRPRC strict mode', ex: error);
      emit(DeveloperSettingsState(wrprcStrictMode: previousValue, hasError: true));
    }
  }

  Future<void> _onWrprcStrictModeToggled(
    DeveloperSettingsWrprcStrictModeToggled event,
    Emitter<DeveloperSettingsState> emit,
  ) async {
    if (!state.canToggleWrprcStrictMode) return;
    final previousValue = state.wrprcStrictMode;
    emit(DeveloperSettingsState(wrprcStrictMode: previousValue, isBusy: true));
    try {
      await _closeProximityRepository.setWrprcStrictMode(enabled: event.enabled);
      emit(DeveloperSettingsState(wrprcStrictMode: event.enabled));
    } catch (error) {
      Fimber.e('Could not save WRPRC strict mode', ex: error);
      emit(DeveloperSettingsState(wrprcStrictMode: previousValue, hasError: true));
    }
  }
}
