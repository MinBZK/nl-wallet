import 'package:rxdart/rxdart.dart';

import '../../../wallet_constants.dart';
import 'wallet_repository.dart';

class MockWalletRepository implements WalletRepository {
  final BehaviorSubject<bool> _locked = BehaviorSubject<bool>.seeded(true);

  MockWalletRepository();

  @override
  Future<bool> isWalletInitialized() async {
    await Future.delayed(kDefaultMockDelay);
    return true;
  }

  @override
  void lockWallet() => _locked.add(true);

  @override
  void unlockWallet(String pin) => _locked.add(pin != kMockPin);

  @override
  Stream<bool> get isLockedStream => _locked.stream.distinct();
}
