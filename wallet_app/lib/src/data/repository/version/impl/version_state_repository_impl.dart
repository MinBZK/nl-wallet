import 'package:wallet_core/core.dart';

import '../../../../domain/model/update/version_state.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../version_state_repository.dart';

class VersionStateRepositoryImpl implements VersionStateRepository {
  final TypedWalletCore _walletCore;
  final Mapper<FlutterVersionState, VersionState> _mapper;

  VersionStateRepositoryImpl(this._walletCore, this._mapper);

  @override
  Stream<VersionState> observeVersionState() => _walletCore.observeVersionState().map(_mapper.map);
}
