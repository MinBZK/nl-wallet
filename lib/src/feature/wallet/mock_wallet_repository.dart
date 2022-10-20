import '../../wallet_constants.dart';
import 'wallet_repository.dart';

class MockWalletRepository implements WalletRepository {
  @override
  Future<bool> isWalletInitialized() async {
    await Future.delayed(kDefaultMockDelay);
    return true;
  }
}
