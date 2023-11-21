import 'package:test/test.dart';
import 'package:wallet_core/core.dart';
import 'package:wallet_mock/src/wallet_core_mock.dart';

void main() {
  late WalletCore walletCore;

  setUp(() {
    walletCore = WalletCoreMock();
  });

  group('WalletCore Initialization', () {
    test('Wallet is not initialized at creation', () async {
      expect(await walletCore.isInitialized(), isFalse);
    });

    test('Calling init initializes the wallet', () async {
      await walletCore.init();
      expect(await walletCore.isInitialized(), isTrue);
    });
  });
}
