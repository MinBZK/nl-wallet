import 'package:test/test.dart';
import 'package:wallet_core/core.dart';
import 'package:wallet_mock/src/log/wallet_event_log.dart';
import 'package:wallet_mock/src/pin/pin_manager.dart';
import 'package:wallet_mock/src/wallet/wallet.dart';
import 'package:wallet_mock/src/wallet_core_mock.dart';

void main() {
  late WalletCoreApi walletCore;

  setUp(() {
    walletCore = WalletCoreMock(PinManager(), Wallet(), WalletEventLog());
  });

  group('WalletCore Initialization', () {
    test('Wallet is not initialized at creation', () async {
      expect(await walletCore.crateApiFullIsInitialized(), isFalse);
    });

    test('Calling init initializes the wallet', () async {
      await walletCore.crateApiFullInit();
      expect(await walletCore.crateApiFullIsInitialized(), isTrue);
    });
  });
}
