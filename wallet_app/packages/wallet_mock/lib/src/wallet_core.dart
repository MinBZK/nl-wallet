import 'log/wallet_event_log.dart';
import 'pin/pin_manager.dart';
import 'wallet/wallet.dart';
import 'wallet_core_for_issuance.dart';
import 'wallet_core_for_signing.dart';
import 'wallet_core_mock.dart';

final PinManager _pinManager = PinManager();
final Wallet _wallet = Wallet();
final WalletEventLog _eventLog = WalletEventLog();

final api = WalletCoreMock(_pinManager, _wallet, _eventLog);

/// Separate issuance implementation, to be merged with [api] once the core [WalletCore] supports issuance.
final issuanceApi = WalletCoreForIssuance(_pinManager, _wallet, _eventLog);
final signingApi = WalletCoreForSigning(_pinManager, _wallet, _eventLog);
