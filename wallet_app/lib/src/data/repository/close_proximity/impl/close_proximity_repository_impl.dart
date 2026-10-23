import 'package:rxdart/rxdart.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../../environment.dart';
import '../../../../domain/model/close_proximity/ble_connection_event.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../../../store/close_proximity_settings_store.dart';
import '../close_proximity_repository.dart';

class CloseProximityRepositoryImpl extends CloseProximityRepository {
  final TypedWalletCore _core;
  final CloseProximitySettingsStore _settings;
  final Mapper<core.CloseProximityDisclosureFlutterUpdate, BleConnectionEvent> _mapper;

  final BehaviorSubject<BleConnectionEvent> _bleConnectionEventSubject = BehaviorSubject<BleConnectionEvent>();

  CloseProximityRepositoryImpl(this._core, this._mapper, this._settings);

  @override
  Future<bool> getWrprcStrictMode() => _settings.getWrprcStrictMode();

  @override
  Future<void> setWrprcStrictMode({required bool enabled}) => _settings.setWrprcStrictMode(enabled: enabled);

  @override
  Future<String> startCloseProximityDisclosure() async {
    final wrprcStrictMode = !Environment.enableWrprcStrictModeToggle || await _settings.getWrprcStrictMode();
    final qrEngagement = await _core.startCloseProximityDisclosure(
      wrprcStrictMode: wrprcStrictMode,
      callback: (update) => _bleConnectionEventSubject.add(_mapper.map(update)),
    );
    _bleConnectionEventSubject.add(BleAdvertising(qrEngagement));
    return qrEngagement;
  }

  @override
  Stream<BleConnectionEvent> observeBleConnectionEvents() => _bleConnectionEventSubject.stream;
}
