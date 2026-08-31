import 'package:test/test.dart';
import 'package:wallet_core/core.dart';
import 'package:wallet_mock/mock.dart';
import 'package:wallet_mock/src/log/wallet_event_log.dart';
import 'package:wallet_mock/src/manager/disclosure_manager.dart';
import 'package:wallet_mock/src/manager/pin_manager.dart';
import 'package:wallet_mock/src/manager/transfer_manager.dart';
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
    final transferManager = TransferManager(pinManager, wallet, walletEventLog);
    walletCore = WalletCoreMock(
      pinManager,
      wallet,
      walletEventLog,
      issuanceManager,
      disclosureManager,
      transferManager,
    );
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

  group('WalletState', () {
    const kPin = '132435';

    test('Wallet is unregistered before a pin is set', () async {
      expect(await walletCore.crateApiFullGetWalletState(), const WalletState.unregistered());
    });

    test('Wallet is empty after registration', () async {
      await walletCore.crateApiFullRegister(pin: kPin);
      expect(await walletCore.crateApiFullGetWalletState(), const WalletState.empty());
    });

    test('Wallet is in the issuance flow while the (empty) wallet awaits the PID', () async {
      await walletCore.crateApiFullRegister(pin: kPin);
      await walletCore.crateApiFullCreatePidIssuanceRedirectUri();
      expect(await walletCore.crateApiFullGetWalletState(), const WalletState.inIssuanceFlow());
    });

    test('Wallet is ready after accepting the PID', () async {
      await walletCore.crateApiFullRegister(pin: kPin);
      await walletCore.crateApiFullCreatePidIssuanceRedirectUri();
      await walletCore.crateApiFullAcceptPidIssuance(pin: kPin);
      expect(await walletCore.crateApiFullGetWalletState(), const WalletState.ready());
    });

    test('Wallet is empty again after cancelling the PID issuance', () async {
      await walletCore.crateApiFullRegister(pin: kPin);
      await walletCore.crateApiFullCreatePidIssuanceRedirectUri();
      await walletCore.crateApiFullCancelSession();
      expect(await walletCore.crateApiFullGetWalletState(), const WalletState.empty());
    });

    test('Wallet is in the issuance flow during PID renewal', () async {
      await walletCore.crateApiFullRegister(pin: kPin);
      await walletCore.crateApiFullCreatePidIssuanceRedirectUri();
      await walletCore.crateApiFullAcceptPidIssuance(pin: kPin);
      await walletCore.crateApiFullCreatePidRenewalRedirectUri();
      expect(await walletCore.crateApiFullGetWalletState(), const WalletState.inIssuanceFlow());
    });
  });

}
