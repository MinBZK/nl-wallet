import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../version_string_repository.dart';

class CoreVersionStringRepository implements VersionStringRepository {
  final TypedWalletCore _walletCore;

  CoreVersionStringRepository(this._walletCore);

  @override
  Future<String> get versionString => _walletCore.getVersionString();
}
