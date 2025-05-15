import 'package:test/test.dart';
import 'package:wallet_core/core.dart';
import 'package:wallet_mock/mock.dart';
import 'package:wallet_mock/src/disclosure_manager.dart';
import 'package:wallet_mock/src/log/wallet_event_log.dart';
import 'package:wallet_mock/src/pin/pin_manager.dart';
import 'package:wallet_mock/src/wallet/wallet.dart';
import 'package:wallet_mock/src/wallet_core_mock.dart';

void main() {
  late WalletCoreApi walletCore;

  setUp(() {
    final pinManager = PinManager();
    final wallet = Wallet();
    final walletEventLog = WalletEventLog();
    final issuanceManager = IssuanceManager(pinManager, wallet, walletEventLog);
    final disclosureManager = DisclosureManager(pinManager, wallet, walletEventLog);
    walletCore = WalletCoreMock(pinManager, wallet, walletEventLog, issuanceManager, disclosureManager);
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
