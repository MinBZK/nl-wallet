import 'package:rxdart/rxdart.dart';

import '../../../wallet_constants.dart';
import 'wallet_repository.dart';

class MockWalletRepository implements WalletRepository {
  var _unlockAttempts = 0;
  final BehaviorSubject<bool> _locked = BehaviorSubject<bool>.seeded(true);
  final BehaviorSubject<bool> _isInitialized = BehaviorSubject<bool>.seeded(true);

  MockWalletRepository();

  @override
  void lockWallet() => _locked.add(true);

  @override
  void unlockWallet(String pin) {
    if (!isInitialized) throw UnsupportedError('Wallet not yet initialized!');
    if (pin == kMockPin) {
      _locked.add(false);
      _unlockAttempts = 0;
    } else {
      _unlockAttempts++;
      if (_unlockAttempts >= kMaxUnlockAttempts) {
        destroyWallet();
      }
    }
  }

  @override
  Future<bool> createWallet(String pin) async {
    if (isInitialized) throw UnsupportedError('Wallet is already initialized');
    await Future.delayed(kDefaultMockDelay);
    _isInitialized.add(true);
    _unlockAttempts = 0;
    return _isInitialized.value;
  }

  @override
  Future<void> destroyWallet() async {
    if (!isInitialized) throw UnsupportedError('Wallet not yet initialized!');
    _isInitialized.add(false);
  }

  @override
  Stream<bool> get isInitializedStream => _isInitialized.stream.distinct();

  bool get isInitialized => _isInitialized.value;

  @override
  Stream<bool> get isLockedStream => _locked.stream.distinct();

  @override
  int get leftoverUnlockAttempts => _isInitialized.value ? kMaxUnlockAttempts - _unlockAttempts : -1;
}
