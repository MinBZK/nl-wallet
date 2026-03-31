import 'package:rxdart/rxdart.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/close_proximity/ble_connection_event.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../close_proximity_repository.dart';

class CloseProximityRepositoryImpl extends CloseProximityRepository {
  final TypedWalletCore _core;
  final Mapper<core.CloseProximityDisclosureFlutterUpdate, BleConnectionEvent> _mapper;

  final BehaviorSubject<BleConnectionEvent> _bleConnectionEventSubject = BehaviorSubject<BleConnectionEvent>();

  CloseProximityRepositoryImpl(this._core, this._mapper);

  @override
  Future<String> startCloseProximityDisclosure() async {
    final qrEngagement = await _core.startCloseProximityDisclosure(
      callback: (update) => _bleConnectionEventSubject.add(_mapper.map(update)),
    );
    _bleConnectionEventSubject.add(BleAdvertising(qrEngagement));
    return qrEngagement;
  }

  @override
  Stream<BleConnectionEvent> observeBleConnectionEvents() => _bleConnectionEventSubject.stream;
}
