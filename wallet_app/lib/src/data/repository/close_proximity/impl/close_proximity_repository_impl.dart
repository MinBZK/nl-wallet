import 'package:fimber/fimber.dart';

import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../close_proximity_repository.dart';

class CloseProximityRepositoryImpl extends CloseProximityRepository {
  final TypedWalletCore _core;

  CloseProximityRepositoryImpl(this._core);

  @override
  Future<String> startCloseProximityDisclosure() => _core.startCloseProximityDisclosure(
    callback: (update) => Fimber.d('CloseProximityDisclosureUpdate: $update'),
  );

  @override
  Future<void> stopCloseProximityDisclosure() {
    throw UnimplementedError();
  }
}
