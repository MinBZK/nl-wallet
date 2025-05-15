import 'disclosure_manager.dart';
import 'issuance_manager.dart';
import 'log/wallet_event_log.dart';
import 'pin/pin_manager.dart';
import 'wallet/wallet.dart';
import 'wallet_core_for_signing.dart';
import 'wallet_core_mock.dart';

final PinManager _pinManager = PinManager();
final Wallet _wallet = Wallet();
final WalletEventLog _eventLog = WalletEventLog();

final _disclosureManager = DisclosureManager(_pinManager, _wallet, _eventLog);
final _issuanceManager = IssuanceManager(_pinManager, _wallet, _eventLog);

final api = WalletCoreMock(_pinManager, _wallet, _eventLog, _issuanceManager, _disclosureManager);

/// Separate signing implementation, to be merged with [api] once the core [WalletCore] supports signing.
final signingApi = WalletCoreForSigning(_pinManager, _wallet, _eventLog);
